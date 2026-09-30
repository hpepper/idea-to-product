use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudfront::config::ProvideCredentials;
use aws_sdk_cloudfront::{Client, Error};
use aws_sdk_eks::Client as EksClient;
use aws_sdk_elasticloadbalancingv2::Client as ElbClient;
use aws_sdk_elasticloadbalancingv2::types::LoadBalancer;
use aws_sdk_route53::Client as Route53Client;
use aws_sdk_route53::types::RrType;
use base64::Engine;
use k8s_openapi::api::networking::v1::Ingress;
use kube::api::{Api, ListParams};
use kube::core::{ApiResource, DynamicObject, GroupVersionKind};
use kube::{Client as K8sClient, Config as K8sConfig};
use secrecy::SecretString;

use rusqlite::Connection;

use std::collections::HashMap;

use sad_xml_sql::db_create_in_mem_db;
use sad_xml_sql::db_retrieval::get_component_by_name;
//use sad_xml_sql::db_update::insert_into_context_model_ignore_duplicates;
//use sad_xml_sql::models;
use sad_xml_sql::db_update::{
    insert_into_component, insert_into_component_relation, insert_into_document,
    insert_into_view_packet,
};
use sad_xml_sql::db_utils::{
    CNC_VIEW_TYPE, CNC_VIEW_TYPE_STYLE_CLIENTSERVER, convert_from_address_to_id,
    create_hardcoded_map,
};
use sad_xml_sql::dump_db_to_xml;

/// Everything a scan of the AWS account needs: the AWS/Kubernetes clients, the in-memory
/// database connection, lookup tables built once up front, and the counters used to mint
/// component/view-packet/relation ids as components are discovered. Bundled into one struct
/// and passed around as `&mut AppContext` so functions don't each need their own list of
/// client and counter parameters.
struct AppContext {
    aws_client: Client,
    elb_client: ElbClient,
    eks_client: EksClient,
    route53_client: Route53Client,
    db_conn: Connection,
    elb_dns_name_to_load_balancer: HashMap<String, LoadBalancer>,
    /// ELB ARN (or, for a non-ARN fallback, the raw hostname) to the component id already
    /// created for it. Several Route53 records/CloudFront origins commonly alias to the same
    /// load balancer, and re-running `investigate_elb`'s Kubernetes discovery (an `aws eks
    /// get-token` shell-out plus a cluster-wide HTTPRoute list) for each one is slow enough to
    /// look like the program has hung, so a repeat hit reuses the cached component instead.
    elb_arn_to_component_id: HashMap<String, u64>,
    type_and_style_to_section_number: HashMap<&'static str, u64>,
    file_id: u64,
    team_id: u64,
    /// Only Kubernetes HTTPRoutes in this namespace are investigated; empty means no filter.
    /// Set via the `--namespace` CLI flag, defaulting to "".
    filtered_namespace: String,
    component_count: u64,
    view_packet_count: u64,
    component_relation_count: u64,
}

/// Lists your CloudFront distributions in the default Region or us-east-1 if a default Region isn't set.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // aws-sdk's rustls stack and kube's rustls stack each pull in rustls without installing a
    // process-wide crypto provider, so install one explicitly before any TLS connection is made.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Installing the rustls ring crypto provider failed.");

    if std::env::args().any(|arg| arg == "version") {
        let version = env!("CARGO_PKG_VERSION");
        println!("Version: {}", version);
        return Ok(());
    }

    let filtered_namespace = parse_filtered_namespace_arg();

    // Prep the Section numbering for the component numbers.
    let type_and_style_to_section_number = create_hardcoded_map();

    // Prep the DB set-up
    let filename = "temporary_sad_aws_account_dump.xml"; //&args[1];
    let file_id = 99; //&args[2];

    let team_id = 0;

    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    db_create_in_mem_db(&db_conn);
    // TOOD get the environment name?
    insert_into_document(
        &db_conn,
        file_id,
        filename,
        "AWS Account Dump",
        "0.1.0",
        "Dump of AWS account data",
    );

    // AWS prep
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    // Resolve credentials once, up front, so an expired SSO session or missing configuration
    // produces one clear, actionable message instead of a raw SDK error nested several layers
    // deep the first time an AWS call happens to need them.
    if let Some(credentials_provider) = config.credentials_provider() {
        if let Err(err) = credentials_provider.provide_credentials().await {
            eprintln!("{}", describe_aws_auth_error(&err));
            std::process::exit(1);
        }
    }

    let aws_client = Client::new(&config);
    let elb_client = ElbClient::new(&config);
    let eks_client = EksClient::new(&config);
    let route53_client = Route53Client::new(&config);

    let elb_dns_name_to_load_balancer = build_elb_dns_name_map(&elb_client).await;

    let mut ctx = AppContext {
        aws_client,
        elb_client,
        eks_client,
        route53_client,
        db_conn,
        elb_dns_name_to_load_balancer,
        elb_arn_to_component_id: HashMap::new(),
        type_and_style_to_section_number,
        file_id,
        team_id,
        filtered_namespace,
        component_count: 1,
        view_packet_count: 1,
        component_relation_count: 1,
    };

    // Itterate over the route53
    itterate_route53_instances(&mut ctx).await;

    // TODO change this to work on a single cloudfrount distribution

    let aws_cloudfront_response = ctx.aws_client.list_distributions().send().await?;
    // Itterate through the cloudfron distributions.
    if let Some(distribution_list) = aws_cloudfront_response.distribution_list() {
        for distribution in distribution_list.items() {
            insert_cloudfront_structure(&mut ctx, distribution.domain_name(), distribution.id()).await;
        }
    }

    dump_db_to_xml(&ctx.db_conn, filename).expect("Dumping the database to XML failed.");
    Ok(())
}

/// Parse `--namespace <name>` (or `--namespace=<name>`) from the command line. Only Kubernetes
/// HTTPRoutes in this namespace are investigated; defaults to "" (no filter) when the flag
/// isn't given.
fn parse_filtered_namespace_arg() -> String {
    let args: Vec<String> = std::env::args().collect();
    for (index, arg) in args.iter().enumerate() {
        if let Some(value) = arg.strip_prefix("--namespace=") {
            return value.to_string();
        }
        if arg == "--namespace" {
            if let Some(value) = args.get(index + 1) {
                return value.to_string();
            }
        }
    }
    String::new()
}

/// Walk the CloudFront distributions and add their structure (distribution, aliases, WebACL,
/// origins and their ELBs, and cache behaviors) to the database as components and relations.
async fn insert_cloudfront_structure(
    ctx: &mut AppContext,
    cloudfront_domain_name: &str,
    distribution_id: &str,
) {
    println!("Id: {cloudfront_domain_name}");

    if get_component_by_name(&ctx.db_conn, cloudfront_domain_name.to_string()).is_ok() {
        println!(
            "DDD CloudFront distribution '{}' already has a component, skipping duplicate creation.",
            cloudfront_domain_name
        );
        return;
    }

    // create a component for the CloudFront distribution and add it to the database
    let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
    let distribution_component_id =
        convert_from_address_to_id(component_addr, "main.rs: cloudfront_structure()");
    let component_title = format!("CloudFront {}", cloudfront_domain_name);
    insert_into_component(
        &ctx.db_conn,
        ctx.file_id,
        distribution_component_id,
        cloudfront_domain_name,
        "Distribution",
        "CloudFront Distribution",
        ctx.team_id,
        &component_title,
    );
    ctx.component_count += 1;

    // TODO create a viewpacket, should this be moved outside the if?
    let view_packet_addr = format!(
        "{}.{}.{}.{}",
        ctx.file_id,
        ctx.type_and_style_to_section_number[CNC_VIEW_TYPE_STYLE_CLIENTSERVER],
        ctx.type_and_style_to_section_number[CNC_VIEW_TYPE],
        ctx.view_packet_count
    );
    let view_packet_id =
        convert_from_address_to_id(view_packet_addr, "main.rs: cloudfront_structure()");
    let context_model_key = "";
    let sort_order = 0;
    let display_key = format!(
        "{}{}{}",
        CNC_VIEW_TYPE, CNC_VIEW_TYPE_STYLE_CLIENTSERVER, ctx.view_packet_count
    );
    let introduction = "TODO fill out introduction";
    let title = format!("CloudFront {}", cloudfront_domain_name);
    insert_into_view_packet(
        &ctx.db_conn,
        distribution_component_id,
        context_model_key,
        ctx.file_id,
        &display_key,
        introduction,
        sort_order,
        ctx.team_id,
        &title,
        CNC_VIEW_TYPE_STYLE_CLIENTSERVER,
        CNC_VIEW_TYPE,
        view_packet_id,
    );
    ctx.view_packet_count += 1;

    // Fetch the full distribution (aliases, origins, WebACL, behaviors) by id. The CloudFront
    // domain name (`cloudfront_domain_name`, e.g. `d111111abcdef8.cloudfront.net`) is a separate
    // identifier from the distribution id and doesn't encode it, so the id has to come from the
    // caller instead of being derived from the domain name.
    let distribution_config = match ctx
        .aws_client
        .get_distribution()
        .id(distribution_id)
        .send()
        .await
    {
        Ok(response) => match response
            .distribution()
            .and_then(|distribution| distribution.distribution_config())
        {
            Some(distribution_config) => distribution_config.clone(),
            None => {
                println!(
                    "WWW CloudFront distribution '{}' has no distribution config. Location: main.rs: cloudfront_structure()",
                    distribution_id
                );
                return;
            }
        },
        Err(err) => {
            println!(
                "WWW Getting CloudFront distribution '{}' failed: {}. Location: main.rs: cloudfront_structure()",
                distribution_id, err
            );
            return;
        }
    };

    // get list of aliases and create components for each alias and add them to the database
    // TODO put in sub function.
    if let Some(aliases) = distribution_config.aliases() {
        for alias in aliases.items() {
            let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
            let alias_component_id =
                convert_from_address_to_id(component_addr, "main.rs: cloudfront_structure()");
            let component_title = format!("CF alias {}", alias);
            insert_into_component(
                &ctx.db_conn,
                ctx.file_id,
                alias_component_id,
                alias,
                "Alias",
                "CloudFront Distribution Alias",
                ctx.team_id,
                &component_title,
            );
            ctx.component_count += 1;

            // TODO create a component relation between the distribution and the alias
            let relation_addr =
                format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
            let relation_id =
                convert_from_address_to_id(relation_addr, "main.rs: cloudfront_structure()");
            let relation_sort_order = 0;
            insert_into_component_relation(
                &ctx.db_conn,
                relation_id,
                relation_sort_order,
                alias_component_id,
                distribution_component_id,
                "connect",
                &display_key,
                "",
                "",
                "",
                "",
            );
            ctx.component_relation_count += 1;
        }
    }

    // if the web_acl is not empty then create it as a component and add it to the database
    let web_acl_id = distribution_config.web_acl_id().unwrap_or("");
    let upstream_component_id = if web_acl_id != "" {
        let web_acl_name = web_acl_id.split('/').nth(2).unwrap_or("Unknown");

        let acl_summary = format!("WebACL: {}", web_acl_id);

        let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
        let web_acl_component_id =
            convert_from_address_to_id(component_addr, "main.rs: cloudfront_structure()");
        let component_title = format!("WebACL {}", web_acl_name);
        insert_into_component(
            &ctx.db_conn,
            ctx.file_id,
            web_acl_component_id,
            web_acl_name,
            "WebACL",
            &acl_summary,
            ctx.team_id,
            &component_title,
        );
        ctx.component_count += 1;

        // create a component relation between the distribution and the web_acl
        let relation_addr =
            format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
        let relation_id =
            convert_from_address_to_id(relation_addr, "main.rs: cloudfront_structure()");
        let relation_sort_order = 0;
        insert_into_component_relation(
            &ctx.db_conn,
            relation_id,
            relation_sort_order,
            distribution_component_id,
            web_acl_component_id,
            "connect",
            &display_key,
            "",
            "",
            "",
            "",
        );
        ctx.component_relation_count += 1;
        web_acl_component_id
    } else {
        distribution_component_id
    };

    // create map for origin id to domain id
    let mut map_origin_id_to_domain_component_id: HashMap<String, u64> = HashMap::new();
    if let Some(origins) = distribution_config.origins() {
        for origin in origins.items() {
            let component_name = format!("origin-{}", origin.id());
            let domain_name = origin.domain_name();

            let origin_type = if origin.s3_origin_config().is_some() {
                // S3 origin: bucket-backed distribution origin.
                "s3 bucket"
            } else if origin.vpc_origin_config.is_some() {
                // VpcOriginConfig: origin lives inside a VPC and is not publicly exposed. CloudFront connects to it privately through a "VPC origin" resource (AWS's newer feature, GA 2024) instead of going over the public internet. Instead of DomainName + ports directly on the origin entry, you reference a VpcOriginId pointing at a VPC origin resource that wraps a private ALB/NLB/EC2 instance.
                "load balancer"
            } else if origin.custom_origin_config.is_some() {
                // CustomOriginConfig: origin is reachable over the public internet (e.g. a public ALB, a DNS name of a third-party server). CloudFront connects to it via its DomainName, using standard internet routing.
                "public access"
            } else {
                "unknown origin"
            };

            let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
            let origin_component_id =
                convert_from_address_to_id(component_addr, "main.rs: cloudfront_structure()");
            let summary = format!("{} - Origin for domain: {}", origin_type, domain_name);
            let component_title = format!("{} {}", origin_type, component_name);
            insert_into_component(
                &ctx.db_conn,
                ctx.file_id,
                origin_component_id,
                &component_name,
                "CloudFront Distribution Origin",
                &summary,
                ctx.team_id,
                &component_title,
            );
            ctx.component_count += 1;
            map_origin_id_to_domain_component_id
                .insert(origin.id().to_string(), origin_component_id);

            // TODO figure out where the lb hooks up: k8s-preproductioninte-16ba1cd511
            // If it contains S3OriginConfig it is an S3 bucket.
            // If it contains CustomOriginConfig is that an ALB?
            // If it container VpcOriginConfig is that an ELB?
            if origin.vpc_origin_config().is_some() {
                investigate_elb_by_hostname(ctx, origin_component_id, domain_name, &display_key)
                    .await;
            }
        }
    };

    // handle if there is a default
    if distribution_config.default_cache_behavior().is_some() {
        let default_cache_behavior = distribution_config.default_cache_behavior().unwrap();
        let target_origin_id = default_cache_behavior.target_origin_id();
        if let Some(origin_component_id) =
            map_origin_id_to_domain_component_id.get(target_origin_id)
        {
            // create a component relation between the distribution and the origin
            let relation_addr =
                format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
            let relation_id =
                convert_from_address_to_id(relation_addr, "main.rs: cloudfront_structure()");
            let relation_sort_order = 0;
            insert_into_component_relation(
                &ctx.db_conn,
                relation_id,
                relation_sort_order,
                upstream_component_id,
                *origin_component_id,
                "connect",
                &display_key,
                "",
                "default",
                "",
                "",
            );
            ctx.component_relation_count += 1;
        } else {
            println!(
                "WWW Origin id '{}' not found in map. Location: main.rs: cloudfront_structure()",
                target_origin_id
            );
        }
    }

    // handle the behaviors
    if let Some(cache_behaviors) = distribution_config.cache_behaviors() {
        for cache_behavior in cache_behaviors.items() {
            let target_origin_id = cache_behavior.target_origin_id();
            let path_pattern = cache_behavior.path_pattern();
            if let Some(origin_component_id) =
                map_origin_id_to_domain_component_id.get(target_origin_id)
            {
                // create a component relation between the distribution and the origin
                let relation_addr =
                    format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
                let relation_id =
                    convert_from_address_to_id(relation_addr, "main.rs: cloudfront_structure()");
                let relation_sort_order = 0;
                insert_into_component_relation(
                    &ctx.db_conn,
                    relation_id,
                    relation_sort_order,
                    upstream_component_id,
                    *origin_component_id,
                    "connect",
                    &display_key,
                    "",
                    path_pattern,
                    "",
                    "",
                );
                ctx.component_relation_count += 1;
            } else {
                println!(
                    "WWW Origin id '{}' not found in map. Location: main.rs: cloudfront_structure()",
                    target_origin_id
                );
            }
        }
    }
}

/// Fetch every Route53 hosted zone in the account (paginated via `marker`/`next_marker`) and
/// add a component and a view packet for each.
async fn itterate_route53_instances(ctx: &mut AppContext) {
    let mut hosted_zones = ctx
        .route53_client
        .list_hosted_zones()
        .into_paginator()
        .items()
        .send();

    while let Some(hosted_zone) = hosted_zones.next().await {
        let hosted_zone = match hosted_zone {
            Ok(hosted_zone) => hosted_zone,
            Err(err) => {
                println!(
                    "WWW Listing Route53 hosted zones failed: {}. Location: main.rs: itterate_route53_instances()",
                    err
                );
                return;
            }
        };

        let zone_name = hosted_zone.name();
        let zone_id = hosted_zone.id().trim_start_matches("/hostedzone/");
        let zone_kind = if hosted_zone
            .config()
            .map(|config| config.private_zone())
            .unwrap_or(false)
        {
            "Private"
        } else {
            "Public"
        };
        let summary = format!("{} Route53 Hosted Zone ({})", zone_kind, zone_id);

        // create a component for the hosted zone and add it to the database
        let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
        let hosted_zone_component_id =
            convert_from_address_to_id(component_addr, "main.rs: itterate_route53_instances()");
        let component_title = format!("Route53 {}",zone_name );
        insert_into_component(
            &ctx.db_conn,
            ctx.file_id,
            hosted_zone_component_id,
            zone_name,
            "Route53 Hosted Zone",
            &summary,
            ctx.team_id,
            &component_title,
        );
        ctx.component_count += 1;

        // create a view packet for the hosted zone
        let view_packet_addr = format!(
            "{}.{}.{}.{}",
            ctx.file_id,
            ctx.type_and_style_to_section_number[CNC_VIEW_TYPE_STYLE_CLIENTSERVER],
            ctx.type_and_style_to_section_number[CNC_VIEW_TYPE],
            ctx.view_packet_count
        );
        let view_packet_id =
            convert_from_address_to_id(view_packet_addr, "main.rs: itterate_route53_instances()");
        let context_model_key = "";
        let sort_order = 0;
        let display_key = format!(
            "{}{}{}",
            CNC_VIEW_TYPE, CNC_VIEW_TYPE_STYLE_CLIENTSERVER, ctx.view_packet_count
        );
        let introduction = "TODO fill out introduction";
        let title = format!("Route53 Hosted Zone {}", zone_name);
        insert_into_view_packet(
            &ctx.db_conn,
            hosted_zone_component_id,
            context_model_key,
            ctx.file_id,
            &display_key,
            introduction,
            sort_order,
            ctx.team_id,
            &title,
            CNC_VIEW_TYPE_STYLE_CLIENTSERVER,
            CNC_VIEW_TYPE,
            view_packet_id,
        );
        ctx.view_packet_count += 1;

        // itterate over the Type A records for the hosted_zone
        itterate_route53_records(ctx, &display_key, zone_id, hosted_zone_component_id).await;
    }
}

/// Fetch every Type A resource record set for a Route53 hosted zone and add a component
/// (related to the hosted zone component) for each. `ListResourceRecordSets` has no SDK
/// paginator, so continuation is done manually via `next_record_name`/`next_record_type`, as
/// AWS's own docs describe.
async fn itterate_route53_records(
    ctx: &mut AppContext,
    display_key: &str,
    hosted_zone_id: &str,
    hosted_zone_component_id: u64,
) {
    let mut start_record_name: Option<String> = None;
    let mut start_record_type: Option<RrType> = None;

    loop {
        let mut request = ctx
            .route53_client
            .list_resource_record_sets()
            .hosted_zone_id(hosted_zone_id);
        if let Some(name) = &start_record_name {
            request = request.start_record_name(name);
        }
        if let Some(record_type) = start_record_type.clone() {
            request = request.start_record_type(record_type);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(err) => {
                println!(
                    "WWW Listing resource record sets for hosted zone '{}' failed: {}. Location: main.rs: itterate_route53_records()",
                    hosted_zone_id, err
                );
                return;
            }
        };

        for record_set in response
            .resource_record_sets()
            .iter()
            .filter(|record_set| record_set.r#type() == &RrType::A)
        {
            // Route53 returns wildcard labels DNS-escaped (e.g. `\052.example.com` instead of
            // `*.example.com`), so unescape it back to `*` for display/storage.
            let record_name = record_set.name().replace("\\052", "*");
            let target_dns_name = match record_set.alias_target() {
                Some(alias_target) => {
                    // Route53 alias targets are FQDNs and come back with a trailing dot (e.g.
                    // `d7lazk7sjwhv3.cloudfront.net.`), but CloudFront's own `domain_name()`
                    // never has one, so strip it here for every downstream comparison/lookup.
                    let dns_name = alias_target.dns_name().trim_end_matches('.');
                    if dns_name.contains(".execute-api.") {
                        println!("DDD API Gateway");
                    } else if dns_name.contains(".cloudfront.net") {
                        println!("DDD CloudFront Distribution");

                        match find_cloudfront_distribution_id(&ctx.aws_client, dns_name).await {
                            Some(distribution_id) => {
                                insert_cloudfront_structure(ctx, dns_name, &distribution_id).await;
                            }
                            None => {
                                println!(
                                    "WWW No CloudFront distribution found with domain name '{}'. Location: main.rs: itterate_route53_records()",
                                    dns_name
                                );
                            }
                        }
                    } else if dns_name.contains(".elb.") {
                        println!("DDD Elastic Load Balancer");
                        investigate_elb_by_hostname(ctx, hosted_zone_component_id, dns_name, display_key).await;
                    } else {
                        println!("DDD Unknown AWS Resource");
                    };
                    dns_name
                }
                None => "",
            };

            let existing_component_option =
                match get_component_by_name(&ctx.db_conn, target_dns_name.to_string()) {
                    Ok(component) => Some(component.id),
                    Err(rusqlite::Error::QueryReturnedNoRows) => None,
                    Err(err) => panic!("expected a component, got an error: {err}"),
                };

            if let Some(existing_component_id) = existing_component_option {
                let relation_addr =
                    format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
                let relation_id = convert_from_address_to_id(
                    relation_addr,
                    "main.rs: itterate_route53_records()",
                );
                let relation_sort_order = 0;
                insert_into_component_relation(
                    &ctx.db_conn,
                    relation_id,
                    relation_sort_order,
                    hosted_zone_component_id,
                    existing_component_id,
                    "connect",
                    display_key,
                    "",
                    &record_name,
                    "",
                    "",
                );
                ctx.component_relation_count += 1;
            } else {
                println!(
                    "DDD No existing component found for alias target '{}', skipping component relation for record '{}'",
                    target_dns_name, record_name
                );
            }
        }

        if response.is_truncated() {
            start_record_name = response.next_record_name().map(String::from);
            start_record_type = response.next_record_type().cloned();
        } else {
            break;
        }
    }
}

/// Look up a CloudFront distribution's id from its domain name (e.g.
/// `d111111abcdef8.cloudfront.net`). A Route53 alias target only gives the domain name, but
/// `get_distribution` needs the id, so this paginates through `list_distributions` until a
/// matching domain name turns up.
async fn find_cloudfront_distribution_id(aws_client: &Client, domain_name: &str) -> Option<String> {
    let mut distributions = aws_client
        .list_distributions()
        .into_paginator()
        .items()
        .send();

    while let Some(distribution) = distributions.next().await {
        let distribution = match distribution {
            Ok(distribution) => distribution,
            Err(err) => {
                println!(
                    "WWW Listing CloudFront distributions failed: {}. Location: main.rs: find_cloudfront_distribution_id()",
                    err
                );
                return None;
            }
        };
        if distribution.domain_name() == domain_name {
            return Some(distribution.id().to_string());
        }
    }

    None
}

/// Walk an AWS SDK error's source chain looking for common auth failure signatures (an
/// expired/invalid SSO session, or no credentials at all) and return an actionable message
/// telling the user what to run, instead of the raw, deeply nested SDK error.
fn describe_aws_auth_error(err: &(dyn std::error::Error + 'static)) -> String {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(error) = current {
        let message = error.to_string();
        if message.contains("Session token not found or invalid")
            || message.contains("UnauthorizedException")
            || message.contains("ExpiredToken")
            || message.contains("InvalidGrantException")
        {
            return "Your AWS SSO session has expired or is invalid. Run `aws sso login` (add `--profile <name>` if you use a named profile), then re-run this program.".to_string();
        }
        if message.contains("the credential provider was not enabled") {
            return "No AWS credentials were found. Run `aws sso login` or `aws configure`, and make sure AWS_PROFILE is set if you use a named profile, then re-run this program.".to_string();
        }
        current = error.source();
    }
    format!("Failed to load AWS credentials: {}", err)
}

/// Fetch every Elastic Load Balancer (ALB/NLB/GWLB) in the account once and index it by
/// DNS name (lowercased), so origins can be matched to their load balancer without an
/// API call per origin.
async fn build_elb_dns_name_map(elb_client: &ElbClient) -> HashMap<String, LoadBalancer> {
    let mut dns_name_to_load_balancer = HashMap::new();
    let mut load_balancers = elb_client
        .describe_load_balancers()
        .into_paginator()
        .items()
        .send();

    while let Some(load_balancer) = load_balancers.next().await {
        let load_balancer = load_balancer.expect("Describing load balancers failed.");
        if let Some(dns_name) = load_balancer.dns_name() {
            dns_name_to_load_balancer.insert(dns_name.to_lowercase(), load_balancer.clone());
        }
    }

    dns_name_to_load_balancer
}

/// Resolve an ELB's ARN from its DNS name (case-insensitively, matching how
/// `build_elb_dns_name_map` indexes `ctx.elb_dns_name_to_load_balancer`), then hand off to
/// `investigate_elb`. Falls back to the hostname itself when no load balancer is found, so the
/// caller still gets a best-effort component instead of silently dropping the origin.
async fn investigate_elb_by_hostname(
    ctx: &mut AppContext,
    origin_component_id: u64,
    hostname: &str,
    display_key: &str,
) {
    let elb_arn = match ctx.elb_dns_name_to_load_balancer.get(&hostname.to_lowercase()) {
        Some(load_balancer) => load_balancer
            .load_balancer_arn()
            .unwrap_or(hostname)
            .to_string(),
        None => {
            println!(
                "WWW No load balancer found with DNS name '{}'. Location: main.rs: investigate_elb_by_hostname()",
                hostname
            );
            hostname.to_string()
        }
    };

    investigate_elb(ctx, origin_component_id, &elb_arn, display_key).await;
}

/// Which AWS Load Balancer Controller resource owns an ELB, identified by which stack tag was
/// found on it. The two tags don't just point at different Kubernetes resource types, they use
/// different tag-value grammars: `service.k8s.aws/stack` (Gateway API) is always
/// `namespace/name`, but `ingress.k8s.aws/stack` is `namespace/name` only for a standalone
/// Ingress; when the ALB is shared across Ingresses via `alb.ingress.kubernetes.io/group.name`,
/// the value is just the group name with no namespace.
#[derive(Clone, Copy)]
enum K8sBackendKind {
    Gateway,
    Ingress,
}

/// Add the ELB behind a CloudFront VPC origin as a component related to the origin
/// component. `elb_arn` identifies the load balancer, resolved by the caller (typically
/// `investigate_elb_by_hostname`, from the origin's domain name). If `elb_arn` is a real ARN,
/// its tags are fetched to recover the Kubernetes Service/Ingress that owns it, when the ELB
/// was created by the AWS Load Balancer Controller.
async fn investigate_elb(
    ctx: &mut AppContext,
    origin_component_id: u64,
    elb_arn: &str,
    display_key: &str,
) {
    // This ELB has already been investigated (a common case: several Route53 records or
    // CloudFront origins alias to the same load balancer), so reuse its component and just
    // connect this origin to it, instead of re-describing it and re-running the (slow)
    // Kubernetes ingress discovery again.
    if let Some(&elb_component_id) = ctx.elb_arn_to_component_id.get(elb_arn) {
        println!(
            "DDD ELB '{}' already investigated, reusing existing component. Location: main.rs: investigate_elb()",
            elb_arn
        );
        let relation_addr = format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
        let relation_id = convert_from_address_to_id(relation_addr, "main.rs: investigate_elb()");
        insert_into_component_relation(
            &ctx.db_conn,
            relation_id,
            0,
            origin_component_id,
            elb_component_id,
            "connect",
            display_key,
            "",
            "",
            "",
            "",
        );
        ctx.component_relation_count += 1;
        return;
    }

    let mut k8s_stack = String::new();
    let mut k8s_backend_kind: Option<K8sBackendKind> = None;
    let mut k8s_cluster = String::new();
    let mut k8s_resource = String::new();

    let mut elb_name = String::new();
    let mut elb_type = String::new();
    let mut elb_scheme = String::new();
    let mut elb_state = String::new();

    println!("DDD elb_arn: {}", elb_arn);

    if elb_arn.starts_with("arn:") {
        match ctx
            .elb_client
            .describe_load_balancers()
            .load_balancer_arns(elb_arn)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(load_balancer) = response.load_balancers().first() {
                    if let Some(name) = load_balancer.load_balancer_name() {
                        elb_name = name.to_string();
                    }
                    if let Some(lb_type) = load_balancer.r#type() {
                        elb_type = lb_type.as_str().to_string();
                    }
                    if let Some(scheme) = load_balancer.scheme() {
                        elb_scheme = scheme.as_str().to_string();
                    }
                    if let Some(state) = load_balancer.state() {
                        if let Some(code) = state.code() {
                            elb_state = code.as_str().to_string();
                        }
                    }
                } else {
                    println!(
                        "WWW No load balancer found for ARN '{}'. Location: main.rs: investigate_elb()",
                        elb_arn
                    );
                }
            }
            Err(err) => {
                println!(
                    "WWW Describing load balancer '{}' failed: {}. Location: main.rs: investigate_elb()",
                    elb_arn, err
                );
            }
        }

        match ctx
            .elb_client
            .describe_tags()
            .resource_arns(elb_arn)
            .send()
            .await
        {
            Ok(response) => {
                for tag in response
                    .tag_descriptions()
                    .iter()
                    .flat_map(|description| description.tags())
                {
                    //println!("DDD tag: {:?}  value: {:?}", tag.key(), tag.value());
                    match tag.key() {
                        Some("ingress.k8s.aws/stack") => {
                            k8s_stack = tag.value().unwrap_or("").to_string();
                            k8s_backend_kind = Some(K8sBackendKind::Ingress);
                            println!("DDD ingress.k8s.aws/stack: {}", k8s_stack)
                        }
                        Some("service.k8s.aws/stack") => {
                            k8s_stack = tag.value().unwrap_or("").to_string();
                            k8s_backend_kind = Some(K8sBackendKind::Gateway);
                            println!("DDD service.k8s.aws/stack: {}", k8s_stack)
                        }
                        Some("elbv2.k8s.aws/cluster") => {
                            k8s_cluster = tag.value().unwrap_or("").to_string()
                        }
                        Some("ingress.k8s.aws/resource") => {
                            k8s_resource = tag.value().unwrap_or("").to_string()
                        }
                        Some("service.k8s.aws/resource") => {
                            k8s_resource = tag.value().unwrap_or("").to_string()
                        }
                        _ => {
                            //println!("DDD unhandled tag key: {:?}", tag.key())
                        }
                    }
                }
            }
            Err(err) => {
                println!(
                    "WWW Describing tags for '{}' failed: {}. Location: main.rs: investigate_elb()",
                    elb_arn, err
                );
            }
        }
        if k8s_stack.is_empty() {
            println!("WWW k8s_stack is empty for elb_arn: {}", elb_arn)
        }
    } else {
        println!("EEE elb_arn not an ARN: {}", elb_arn);
    }

    let component_name = if elb_name.is_empty() {
        elb_arn.rsplit('/').next().unwrap_or(elb_arn).to_string()
    } else {
        elb_name
    };

    let summary = {
        let mut details = Vec::new();
        if !elb_type.is_empty() {
            details.push(format!("type: {}", elb_type));
        }
        if !elb_scheme.is_empty() {
            details.push(format!("scheme: {}", elb_scheme));
        }
        if !elb_state.is_empty() {
            details.push(format!("state: {}", elb_state));
        }
        if details.is_empty() {
            "ELB".to_string()
        } else {
            format!("ELB ({})", details.join(", "))
        }
    };

    // create a component for the ELB behind the VPC origin and add it to the database
    let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
    let elb_component_id = convert_from_address_to_id(component_addr, "main.rs: investigate_elb()");
    let component_title = format!("ELB {}", component_name);
    insert_into_component(
        &ctx.db_conn,
        ctx.file_id,
        elb_component_id,
        &component_name,
        "",
        &summary,
        ctx.team_id,
        &component_title,
    );
    ctx.component_count += 1;
    ctx.elb_arn_to_component_id
        .insert(elb_arn.to_string(), elb_component_id);

    // create a component relation between the origin and the ELB
    let relation_addr = format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
    let relation_id = convert_from_address_to_id(relation_addr, "main.rs: investigate_elb()");
    let relation_sort_order = 0;
    insert_into_component_relation(
        &ctx.db_conn,
        relation_id,
        relation_sort_order,
        origin_component_id,
        elb_component_id,
        "connect",
        display_key,
        "",
        "",
        "",
        "",
    );
    ctx.component_relation_count += 1;

    // Connect to the Kubernetes API server of the EKS cluster that owns this ELB (identified by
    // the `elbv2.k8s.aws/cluster` tag), then dispatch to the discovery path that matches which
    // AWS Load Balancer Controller resource actually owns it: `K8sBackendKind::Gateway` stacks are
    // investigated via Gateway API HTTPRoutes, `K8sBackendKind::Ingress` stacks via classic
    // `networking.k8s.io/v1` Ingresses. The cluster's endpoint and CA certificate come from
    // `eks:DescribeCluster`; the bearer token is minted by shelling out to `aws eks get-token`,
    // since it's a short-lived (~15 minute) presigned STS token rather than something an API call
    // can hand back directly.
    println!("DDD investigate_k8s_ingresses for k8s_stack: {}", k8s_stack);

    let Some(backend_kind) = k8s_backend_kind else {
        println!(
            "WWW No ingress.k8s.aws/stack or service.k8s.aws/stack tag found, skipping Kubernetes ingress investigation. Location: main.rs: investigate_elb()"
        );
        return;
    };

    let Some(k8s_client) = build_k8s_client(&ctx.eks_client, &k8s_cluster).await else {
        return;
    };

    match backend_kind {
        K8sBackendKind::Gateway => {
            investigate_gateway_http_routes(ctx, elb_component_id, display_key, k8s_client, &k8s_stack)
                .await;
        }
        K8sBackendKind::Ingress => {
            investigate_ingress_backends(ctx, elb_component_id, display_key, k8s_client, &k8s_stack)
                .await;
        }
    }
}

/// Add a Gateway API stack's HTTPRoutes as components related to the ELB component. HTTPRoute
/// isn't a core Kubernetes type known to `k8s-openapi`, so it's fetched as a `DynamicObject`
/// keyed by its `gateway.networking.k8s.io/v1` GVK. `k8s_stack` (from `service.k8s.aws/stack`)
/// is always `namespace/name`.
async fn investigate_gateway_http_routes(
    ctx: &mut AppContext,
    elb_component_id: u64,
    display_key: &str,
    k8s_client: K8sClient,
    k8s_stack: &str,
) {
    let (elb_namespace, elb_container_name) = k8s_stack.split_once("/").unwrap_or(("", k8s_stack));
    if elb_namespace.is_empty() {
        println!("EEE k8s_stack could not be split on '/': {}", k8s_stack)
    };

    //  ###
    //   #     #    #   ####   #####   ######   ####    ####
    //   #     ##   #  #    #  #    #  #       #       #
    //   #     # #  #  #       #    #  #####    ####    ####
    //   #     #  # #  #  ###  #####   #            #       #
    //   #     #   ##  #    #  #   #   #       #    #  #    #
    //  ###    #    #   ####   #    #  ######   ####    ####

    let http_route_resource = ApiResource::from_gvk_with_plural(
        &GroupVersionKind::gvk("gateway.networking.k8s.io", "v1", "HTTPRoute"),
        "httproutes",
    );
    let http_routes: Api<DynamicObject> = Api::all_with(k8s_client, &http_route_resource);
    let http_route_list = match http_routes.list(&ListParams::default()).await {
        Ok(list) => list,
        Err(err) => {
            println!(
                "EEE Listing HTTPRoutes for stack '{}' failed: {}. Location: main.rs: investigate_gateway_http_routes()",
                k8s_stack, err
            );
            return;
        }
    };

    for http_route in http_route_list {
        let meata_namespace = http_route
            .metadata
            .namespace
            .as_deref()
            .unwrap_or("default");
        let meta_name = http_route.metadata.name.as_deref().unwrap_or("Unknown");

        /*
        println!(
            "DDD httproute meta_ns: {} meta_name: {}",
            namespace, meta_name
        );
         */

        if ctx.filtered_namespace.is_empty() || meata_namespace == ctx.filtered_namespace {
            println!(
                "DDD httproute meta_ns: {} meta_name: {}",
                meata_namespace, meta_name
            );
            // get all hostnames under spec.hostnames
            let mut hostnames: Vec<&str> = http_route.data["spec"]["hostnames"]
                .as_array()
                .map(|hostnames| {
                    hostnames
                        .iter()
                        .filter_map(|hostname| hostname.as_str())
                        .collect()
                })
                .unwrap_or_default();
            if hostnames.is_empty() {
                hostnames.push("TODO fix empty hostnames");
            }

            // get parent_name spec.parentRefs[].name and parent_namespace spec.parentRefs[].namespace
            let parent_refs: Vec<(&str, &str)> = http_route.data["spec"]["parentRefs"]
                .as_array()
                .map(|parent_refs| {
                    parent_refs
                        .iter()
                        .map(|parent_ref| {
                            let parent_name = parent_ref["name"].as_str().unwrap_or("Unknown");
                            let parent_namespace =
                                parent_ref["namespace"].as_str().unwrap_or(meata_namespace);
                            (parent_namespace, parent_name)
                        })
                        .collect()
                })
                .unwrap_or_default();

            // TODO verify the parent_refs ns is the elb ns, and parent_name is the name of the ingress

            let correct_ns_and_container = if let Some((parent_namespace, parent_container_name)) =
                parent_refs.first()
            {
                let parent_full_container_name = format!("{}-nginx", parent_container_name);
                if parent_full_container_name == elb_container_name
                    && *parent_namespace == elb_namespace
                {
                    println!("DDD NS and container names fit");
                    true
                } else {
                    println!(
                        "DDD NS and container names doe NOT fit. parent NS {}, parent_container name: {}",
                        parent_namespace, parent_full_container_name
                    );
                    false
                }
            } else {
                false
            };

            if correct_ns_and_container {
                // The HTTPRoute parentRefs references the same container name and namespace as the ELB, so this is the correct httproute.

                println!("DDD Correct HTTPRoute found.");

                // get backend_ref_name spec.rules[].backendRefs[].name and its kind
                // spec.rules[].backendRefs[].kind (defaults to "Service" per the Gateway API spec
                // when omitted). There can actually be multiple backendrefs according to the json.
                let backend_refs: Vec<(&str, &str)> = http_route.data["spec"]["rules"]
                    .as_array()
                    .map(|rules| {
                        rules
                            .iter()
                            .flat_map(|rule| rule["backendRefs"].as_array().into_iter().flatten())
                            .filter_map(|backend_ref| {
                                let backend_ref_name = backend_ref["name"].as_str()?;
                                let backend_ref_kind =
                                    backend_ref["kind"].as_str().unwrap_or("Service");
                                Some((backend_ref_kind, backend_ref_name))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                println!(
                    "DDD httproute {} backend_refs: {:?}",
                    meta_name, backend_refs
                );
                if backend_refs.len() > 1 {
                    println!(
                        "WWW backendRefs has more than one entry, actual: {}",
                        backend_refs.len()
                    );
                }
                if let Some((backend_kind, backend_name)) = backend_refs.first() {
                    if *backend_kind != "Service" {
                        println!(
                            "EEE unexpected backendRefs kind(expected Service) found: {} for name: {}",
                            backend_kind, backend_name
                        );
                    } else {
                        record_backend_relation(
                            ctx,
                            elb_component_id,
                            display_key,
                            backend_kind,
                            backend_name,
                            &hostnames,
                        );
                    }
                }
            }
        }
    }
}

/// Add a classic-Ingress stack's routes as components related to the ELB component, using the
/// typed `networking.k8s.io/v1 Ingress` (a well-known core type, unlike `HTTPRoute`). Matching
/// depends on `ingress.k8s.aws/stack`'s grammar: `k8s_stack` is `namespace/name` for a
/// standalone Ingress, but just the IngressGroup name (no namespace) when the ALB is shared
/// across Ingresses via the `alb.ingress.kubernetes.io/group.name` annotation, since a group can
/// span multiple namespaces. Every `(host, backend)` pair from `spec.rules[].http.paths[]` is
/// recorded, unlike the Gateway path's "first backend wins": path-based routing (`/api` to one
/// Service, `/` to another, on the same host) is common in classic Ingress and collapsing it
/// would lose real topology.
async fn investigate_ingress_backends(
    ctx: &mut AppContext,
    elb_component_id: u64,
    display_key: &str,
    k8s_client: K8sClient,
    k8s_stack: &str,
) {
    let ingresses: Api<Ingress> = Api::all(k8s_client);
    let ingress_list = match ingresses.list(&ListParams::default()).await {
        Ok(list) => list,
        Err(err) => {
            println!(
                "EEE Listing Ingresses for stack '{}' failed: {}. Location: main.rs: investigate_ingress_backends()",
                k8s_stack, err
            );
            return;
        }
    };

    let standalone_namespace_and_name = k8s_stack.split_once('/');
    let mut matched_any = false;

    for ingress in ingress_list {
        let ingress_namespace = ingress.metadata.namespace.as_deref().unwrap_or("default");
        let ingress_name = ingress.metadata.name.as_deref().unwrap_or("Unknown");

        if !ctx.filtered_namespace.is_empty() && ingress_namespace != ctx.filtered_namespace {
            continue;
        }

        let stack_matches = match standalone_namespace_and_name {
            Some((namespace, name)) => ingress_namespace == namespace && ingress_name == name,
            None => ingress
                .metadata
                .annotations
                .as_ref()
                .and_then(|annotations| {
                    annotations.get("alb.ingress.kubernetes.io/group.name")
                })
                .is_some_and(|group_name| group_name == k8s_stack),
        };
        if !stack_matches {
            continue;
        }
        matched_any = true;

        println!(
            "DDD Ingress {}/{} matches stack '{}'",
            ingress_namespace, ingress_name, k8s_stack
        );

        let Some(rules) = ingress.spec.as_ref().and_then(|spec| spec.rules.as_ref()) else {
            continue;
        };

        for rule in rules {
            let Some(host) = rule.host.as_deref() else {
                continue;
            };
            let Some(paths) = rule.http.as_ref().map(|http| &http.paths) else {
                continue;
            };

            for path in paths {
                match path.backend.service.as_ref() {
                    Some(service_backend) => {
                        record_backend_relation(
                            ctx,
                            elb_component_id,
                            display_key,
                            "Service",
                            &service_backend.name,
                            &[host],
                        );
                    }
                    None => {
                        println!(
                            "EEE Ingress {}/{} path backend has no Service (a custom resource backend?), skipping. Location: main.rs: investigate_ingress_backends()",
                            ingress_namespace, ingress_name
                        );
                    }
                }
            }
        }
    }

    if !matched_any {
        println!(
            "WWW No Ingress found matching stack '{}'. Location: main.rs: investigate_ingress_backends()",
            k8s_stack
        );
    }
}

/// Look up (or create) the component for a Kubernetes Service backend, then add one relation
/// per hostname connecting it to the ELB component. Shared by both the Gateway API (HTTPRoute)
/// and classic Ingress discovery paths, since both ultimately resolve to "this hostname on the
/// ELB is routed to this Service."
fn record_backend_relation(
    ctx: &mut AppContext,
    elb_component_id: u64,
    display_key: &str,
    backend_kind: &str,
    backend_name: &str,
    hostnames: &[&str],
) {
    let component_name = format!("{}-{}", backend_name, backend_kind);

    let backend_component_id = match get_component_by_name(&ctx.db_conn, component_name.clone()) {
        Ok(component) => component.id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let new_component_id = ctx.component_count;
            let component_addr = format!("{}.0.0.{}", ctx.file_id, new_component_id);
            let backend_id =
                convert_from_address_to_id(component_addr, "main.rs: record_backend_relation()");

            let summary = format!("{}: {}", backend_name, backend_kind);
            // TODO figure out what type of backend this is.
            let component_title = format!("{} {}", backend_kind, backend_name);
            insert_into_component(
                &ctx.db_conn,
                ctx.file_id,
                backend_id,
                &component_name,
                "",
                &summary,
                ctx.team_id,
                &component_title,
            );
            ctx.component_count += 1;
            backend_id
        }
        Err(err) => panic!("expected a component, got an error: {err}"),
    };

    for hostname in hostnames.iter().copied() {
        let relation_addr = format!("{}.{}.{}.{}", ctx.file_id, 0, 2, ctx.component_relation_count);
        let relation_id =
            convert_from_address_to_id(relation_addr.clone(), "main.rs: record_backend_relation()");
        println!(
            "DDD Adding component relation for component id: {} hostname: '{}'",
            relation_addr, hostname
        );
        insert_into_component_relation(
            &ctx.db_conn,
            relation_id,
            0,
            elb_component_id,
            backend_component_id,
            "connect",
            display_key,
            "",
            hostname,
            "",
            "",
        );
        ctx.component_relation_count += 1;
    }
}

/// Build a `kube::Client` for an EKS cluster's API server. The endpoint and CA certificate
/// come from `eks:DescribeCluster`; the bearer token is minted by shelling out to
/// `aws eks get-token`, since it's a short-lived (~15 minute) presigned STS token rather than
/// something an API call can hand back directly. Returns `None` (after printing a warning) if
/// `k8s_cluster_name` is empty or on any failure along the way.
async fn build_k8s_client(eks_client: &EksClient, k8s_cluster_name: &str) -> Option<K8sClient> {
    if k8s_cluster_name.is_empty() {
        println!("DDD EKS cluster name not defined, skipped build_k8s_client");
        return None;
    }

    let cluster = match eks_client
        .describe_cluster()
        .name(k8s_cluster_name)
        .send()
        .await
    {
        Ok(response) => match response.cluster() {
            Some(cluster) => cluster.clone(),
            None => {
                println!(
                    "WWW EKS cluster '{}' not found. Location: main.rs: build_k8s_client()",
                    k8s_cluster_name
                );
                return None;
            }
        },
        Err(err) => {
            println!(
                "WWW Describing EKS cluster '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            );
            return None;
        }
    };

    let Some(endpoint) = cluster.endpoint() else {
        println!(
            "WWW EKS cluster '{}' has no API endpoint. Location: main.rs: build_k8s_client()",
            k8s_cluster_name
        );
        return None;
    };
    let cluster_url: http::Uri = match endpoint.parse() {
        Ok(url) => url,
        Err(err) => {
            println!(
                "WWW Parsing endpoint '{}' for cluster '{}' failed: {}. Location: main.rs: build_k8s_client()",
                endpoint, k8s_cluster_name, err
            );
            return None;
        }
    };

    let Some(ca_data) = cluster.certificate_authority().and_then(|ca| ca.data()) else {
        println!(
            "WWW EKS cluster '{}' has no certificate authority data. Location: main.rs: build_k8s_client()",
            k8s_cluster_name
        );
        return None;
    };
    let ca_pem = match base64::engine::general_purpose::STANDARD.decode(ca_data) {
        Ok(pem) => pem,
        Err(err) => {
            println!(
                "WWW Decoding certificate authority data for '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            );
            return None;
        }
    };
    let root_cert = match pem::parse_many(&ca_pem) {
        Ok(pems) => pems
            .into_iter()
            .filter(|pem| pem.tag() == "CERTIFICATE")
            .map(|pem| pem.into_contents())
            .collect::<Vec<Vec<u8>>>(),
        Err(err) => {
            println!(
                "WWW Parsing certificate authority data for '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            );
            return None;
        }
    };

    let bearer_token = match get_eks_bearer_token(k8s_cluster_name) {
        Ok(token) => token,
        Err(err) => {
            println!(
                "WWW Getting a bearer token for EKS cluster '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            );
            return None;
        }
    };

    let mut k8s_config = K8sConfig::new(cluster_url);
    k8s_config.root_cert = Some(root_cert);
    k8s_config.auth_info.token = Some(SecretString::from(bearer_token));
    // `kube`'s default Config has no connect/read timeout at all, so an EKS cluster whose API
    // endpoint isn't reachable from here (e.g. a private endpoint with no VPN into that VPC)
    // hangs forever instead of failing like an unreachable one that at least gets a fast
    // connection-refused. Bound both so a dead cluster surfaces as a warning, not a hang.
    k8s_config.connect_timeout = Some(std::time::Duration::from_secs(10));
    k8s_config.read_timeout = Some(std::time::Duration::from_secs(30));

    match K8sClient::try_from(k8s_config) {
        Ok(client) => Some(client),
        Err(err) => {
            println!(
                "WWW Building a Kubernetes client for '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            );
            None
        }
    }
}

/// Get a short-lived Kubernetes bearer token for an EKS cluster by shelling out to
/// `aws eks get-token`, which mints it as a presigned STS `GetCallerIdentity` request (the
/// `aws-iam-authenticator` scheme) rather than something an AWS API call returns directly.
fn get_eks_bearer_token(cluster_name: &str) -> Result<String, String> {
    let output = std::process::Command::new("aws")
        .args([
            "eks",
            "get-token",
            "--cluster-name",
            cluster_name,
            "--output",
            "json",
        ])
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let token_response: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())?;
    token_response["status"]["token"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "missing status.token in `aws eks get-token` output".to_string())
}
