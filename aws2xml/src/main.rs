use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudfront::config::ProvideCredentials;
use aws_sdk_cloudfront::Client;
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
use sad_xml_sql::db_update::{
    insert_into_component, insert_into_component_relation, insert_into_document,
    insert_into_view_packet,
};
use sad_xml_sql::db_utils::{
    CNC_VIEW_TYPE, CNC_VIEW_TYPE_STYLE_CLIENTSERVER, convert_from_address_to_id,
    create_hardcoded_map,
};
use sad_xml_sql::dump_db_to_xml;


use aws2xml::{emit_log, init_logger, init_metrics, init_tracer, service_log};
use opentelemetry::{
    KeyValue,
    trace::{Span, Status, Tracer},
};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::logs::Severity;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::{Arc, OnceLock};
use tracing::Level;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::*;

static LOG_PROVIDER: OnceLock<SdkLoggerProvider> = OnceLock::new();
static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();

/// Shared state for one account scan: clients, database, lookup tables and id counters.
struct AppContext {
    aws_client: Client,
    elb_client: ElbClient,
    eks_client: EksClient,
    route53_client: Route53Client,
    db_conn: Connection,
    elb_dns_name_to_load_balancer: HashMap<String, LoadBalancer>,
    /// ELB hostname to its component id. Many records alias the same ELB, and repeating the
    /// slow Kubernetes discovery for each one looks like a hang.
    elb_hostname_to_component_id: HashMap<String, u64>,
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


/// Messages below this severity are dropped.
/// Set with `AWS2XML_LOG_LEVEL` (debug, info, warn, error); defaults to info.
static MIN_LOG_SEVERITY: OnceLock<Severity> = OnceLock::new();

fn read_min_log_severity() -> Severity {
    match std::env::var("AWS2XML_LOG_LEVEL")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "debug" => Severity::Debug,
        "warn" => Severity::Warn,
        "error" => Severity::Error,
        _ => Severity::Info,
    }
}

/// Print one readable line to stderr and send the message to the OTel log backend.
/// Messages are only printed until `main` has stored the logger in `LOG_PROVIDER`.
fn log_at(severity: Severity, msg: impl Into<String>) {
    if severity < *MIN_LOG_SEVERITY.get_or_init(read_min_log_severity) {
        return;
    }
    let msg = msg.into();
    eprintln!("{} {}", severity.name(), msg);
    // The event shows the message inside the current span in Tempo. Warnings mark the span as
    // failed so skipped branches stand out; error events do that automatically.
    match severity {
        Severity::Debug => tracing::debug!("{msg}"),
        Severity::Warn => {
            tracing::warn!("{msg}");
            tracing::Span::current().set_status(Status::error(msg.clone()));
        }
        Severity::Error => tracing::error!("{msg}"),
        _ => tracing::info!("{msg}"),
    }
    if let Some(p) = LOG_PROVIDER.get() {
        service_log(p, severity, msg);
    }
}

fn log_debug(msg: impl Into<String>) {
    log_at(Severity::Debug, msg);
}

fn log_info(msg: impl Into<String>) {
    log_at(Severity::Info, msg);
}

fn log_warn(msg: impl Into<String>) {
    log_at(Severity::Warn, msg);
}

fn log_error(msg: impl Into<String>) {
    log_at(Severity::Error, msg);
}

/// Export the `#[tracing::instrument]` spans of this crate through OTel.
/// Spans from dependencies such as the AWS SDK are filtered out to keep traces readable.
fn init_tracing() {
    let provider = match init_tracer("aws2xml", env!("CARGO_PKG_VERSION")) {
        Ok(provider) => provider,
        Err(err) => {
            log_warn(format!("Starting the OTel tracer failed, continuing without traces: {err}"));
            return;
        }
    };
    let only_this_crate = Targets::new().with_target("aws2xml", Level::TRACE);
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(provider.tracer("aws2xml"))
        .with_filter(only_this_crate);
    tracing_subscriber::registry().with(otel_layer).init();
    let _ = TRACER_PROVIDER.set(provider);
}

/// Flush batched spans and log records; without this the last ones before exit are lost.
fn shutdown_telemetry() {
    if let Some(p) = TRACER_PROVIDER.get() {
        if let Err(err) = p.shutdown() {
            eprintln!("ERROR Shutting down the OTel tracer failed: {err}");
        }
    }
    if let Some(p) = LOG_PROVIDER.get() {
        if let Err(err) = p.shutdown() {
            eprintln!("ERROR Shutting down the OTel logger failed: {err}");
        }
    }
}

/// Scan the AWS account's Route53 hosted zones and CloudFront distributions, follow them to
/// their load balancers and Kubernetes backends, and dump the result as a SAD XML document.
#[tokio::main]
async fn main() {
    // aws-sdk's rustls stack and kube's rustls stack each pull in rustls without installing a
    // process-wide crypto provider, so install one explicitly before any TLS connection is made.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Installing the rustls ring crypto provider failed.");

    if std::env::args().any(|arg| arg == "version") {
        let version = env!("CARGO_PKG_VERSION");
        println!("Version: {}", version);
        return;
    }

    let _ = LOG_PROVIDER.set(init_logger("aws2xml", env!("CARGO_PKG_VERSION")));
    init_tracing();
    let result = run().await;
    finish_run(result);
}

/// Flush telemetry, then exit with status 1 if `run` failed.
/// Exiting here instead of inside `run` lets the root span close and be exported.
fn finish_run(result: Result<(), Box<dyn std::error::Error>>) {
    if let Err(err) = &result {
        log_error(err.to_string());
    }
    shutdown_telemetry();
    if result.is_err() {
        std::process::exit(1);
    }
}

/// The account scan itself, split from `main` so that every early return still reaches
/// `shutdown_telemetry`. Its span is the root of the trace.
#[tracing::instrument(skip_all, err)]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let filtered_namespace = parse_filtered_namespace_arg();

    // View packet addresses embed the SAD section number of their view type and style.
    let type_and_style_to_section_number = create_hardcoded_map();

    // TODO: Read the output filename and file id from the command line.
    let filename = "temporary_sad_aws_account_dump.xml";
    let file_id = 99;

    let team_id = 0;

    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    db_create_in_mem_db(&db_conn);
    // TODO: Include the AWS environment name in the document metadata.
    insert_into_document(
        &db_conn,
        file_id,
        filename,
        "AWS Account Dump",
        "0.1.0",
        "Dump of AWS account data",
    );

    // CloudFront and Route53 are global services, so us-east-1 is a safe fallback Region.
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    // Fail early with a clear message instead of a nested SDK error on the first AWS request.
    if let Some(credentials_provider) = config.credentials_provider() {
        if let Err(err) = credentials_provider.provide_credentials().await {
            return Err(describe_aws_auth_error(&err).into());
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
        elb_hostname_to_component_id: HashMap::new(),
        type_and_style_to_section_number,
        file_id,
        team_id,
        filtered_namespace,
        component_count: 1,
        view_packet_count: 1,
        component_relation_count: 1,
    };

    itterate_route53_instances(&mut ctx).await;

    // TODO: Support scanning a single CloudFront distribution.

    let aws_cloudfront_response = ctx.aws_client.list_distributions().send().await?;
    if let Some(distribution_list) = aws_cloudfront_response.distribution_list() {
        for distribution in distribution_list.items() {
            insert_cloudfront_structure(&mut ctx, distribution.domain_name(), distribution.id())
                .await;
        }
    }

    dump_db_to_xml(&ctx.db_conn, filename).expect("Dumping the database to XML failed.");
    Ok(())
}

/// Parse `--namespace <name>` or `--namespace=<name>`. Returns "" (no filter) when absent.
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
#[tracing::instrument(skip(ctx))]
async fn insert_cloudfront_structure(
    ctx: &mut AppContext,
    cloudfront_domain_name: &str,
    distribution_id: &str,
) {
    log_debug(format!("Cloudfront id: {cloudfront_domain_name}"));

    if get_component_by_name(&ctx.db_conn, cloudfront_domain_name.to_string()).is_ok() {
        log_debug(format!(
            "CloudFront distribution '{}' already has a component, skipping duplicate creation.",
            cloudfront_domain_name
        ));
        return;
    }

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

    // TODO: Decide whether view packet creation belongs outside the duplicate check above.
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

    // The domain name does not encode the distribution id, so the caller supplies it.
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
                log_warn(format!(
                    "CloudFront distribution '{}' has no distribution config. Location: main.rs: cloudfront_structure()",
                    distribution_id
                ));
                return;
            }
        },
        Err(err) => {
            log_warn(format!(
                "Getting CloudFront distribution '{}' failed: {}. Location: main.rs: cloudfront_structure()",
                distribution_id, err
            ));
            return;
        }
    };

    // TODO: Move alias handling into its own function.
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

            let relation_addr = format!(
                "{}.{}.{}.{}",
                ctx.file_id, 0, 2, ctx.component_relation_count
            );
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

    // Requests pass through the WebACL before reaching an origin, so an attached WebACL
    // becomes the upstream component that the origins connect to.
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

        let relation_addr = format!(
            "{}.{}.{}.{}",
            ctx.file_id, 0, 2, ctx.component_relation_count
        );
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

    // Cache behaviors reference origins by origin id, so map each origin id to the component
    // id that the behavior relations connect to.
    let mut map_origin_id_to_domain_component_id: HashMap<String, u64> = HashMap::new();
    if let Some(origins) = distribution_config.origins() {
        for origin in origins.items() {
            let component_name = format!("origin-{}", origin.id());
            let domain_name = origin.domain_name();

            let origin_type = if origin.s3_origin_config().is_some() {
                "s3 bucket"
            } else if origin.vpc_origin_config.is_some() {
                // A VPC origin is not publicly exposed. CloudFront reaches it privately through a
                // VPC origin resource (GA 2024) that wraps a private ALB, NLB or EC2 instance.
                "load balancer"
            } else if origin.custom_origin_config.is_some() {
                // A custom origin is reachable over the public internet by its domain name, for
                // example a public ALB or a third-party server.
                "public access"
            } else {
                "unknown origin"
            };

            // A VPC origin is the ELB itself, so let the cache behaviors connect straight to the
            // ELB component instead of creating a separate origin component for it.
            if origin.vpc_origin_config().is_some() {
                investigate_elb_by_hostname(ctx, upstream_component_id, domain_name, &display_key)
                    .await;
                if let Some(&elb_component_id) = ctx.elb_hostname_to_component_id.get(domain_name) {
                    map_origin_id_to_domain_component_id
                        .insert(origin.id().to_string(), elb_component_id);
                    continue;
                }
            }

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

            // TODO: Find where load balancer k8s-preproductioninte-16ba1cd511 connects. A custom
            // origin can also point at a public ALB, which is not followed to its ELB yet.
        }
    };

    // The default cache behavior handles requests that match no cache behavior path pattern.
    if distribution_config.default_cache_behavior().is_some() {
        let default_cache_behavior = distribution_config.default_cache_behavior().unwrap();
        let target_origin_id = default_cache_behavior.target_origin_id();
        if let Some(origin_component_id) =
            map_origin_id_to_domain_component_id.get(target_origin_id)
        {
            let relation_addr = format!(
                "{}.{}.{}.{}",
                ctx.file_id, 0, 2, ctx.component_relation_count
            );
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
            log_warn(format!(
                "Origin id '{}' not found in map. Location: main.rs: cloudfront_structure()",
                target_origin_id
            ));
        }
    }

    if let Some(cache_behaviors) = distribution_config.cache_behaviors() {
        for cache_behavior in cache_behaviors.items() {
            let target_origin_id = cache_behavior.target_origin_id();
            let path_pattern = cache_behavior.path_pattern();
            if let Some(origin_component_id) =
                map_origin_id_to_domain_component_id.get(target_origin_id)
            {
                let relation_addr = format!(
                    "{}.{}.{}.{}",
                    ctx.file_id, 0, 2, ctx.component_relation_count
                );
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
                log_warn(format!(
                    "Origin id '{}' not found in map. Location: main.rs: cloudfront_structure()",
                    target_origin_id
                ));
            }
        }
    }
}

/// Fetch every Route53 hosted zone in the account (paginated via `marker`/`next_marker`) and
/// add a component and a view packet for each.
#[tracing::instrument(skip_all)]
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
                log_warn(format!(
                    "Listing Route53 hosted zones failed: {}. Location: main.rs: itterate_route53_instances()",
                    err
                ));
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

        let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
        let hosted_zone_component_id =
            convert_from_address_to_id(component_addr, "main.rs: itterate_route53_instances()");
        let component_title = format!("Route53 {}", zone_name);
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

        itterate_route53_records(ctx, &display_key, zone_id, hosted_zone_component_id).await;
    }
}

/// Add a component for each Type A record in a hosted zone. `ListResourceRecordSets` has no
/// SDK paginator, so pagination is manual.
#[tracing::instrument(skip(ctx))]
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
                log_warn(format!(
                    "Listing resource record sets for hosted zone '{}' failed: {}. Location: main.rs: itterate_route53_records()",
                    hosted_zone_id, err
                ));
                return;
            }
        };

        for record_set in response
            .resource_record_sets()
            .iter()
            .filter(|record_set| record_set.r#type() == &RrType::A)
        {
            // Route53 returns wildcard labels DNS-escaped (`\052.example.com` instead of
            // `*.example.com`), so restore the `*` for display and storage.
            let record_name = record_set.name().replace("\\052", "*");
            let target_dns_name = match record_set.alias_target() {
                Some(alias_target) => {
                    // Route53 alias targets are FQDNs with a trailing dot (`XXX.cloudfront.net.`),
                    // which the CloudFront and ELB lookups do not expect.
                    let dns_name = alias_target.dns_name().trim_end_matches('.');
                    if dns_name.contains(".execute-api.") {
                        log_warn("TODO implement API Gateway");
                    } else if dns_name.contains(".cloudfront.net") {
                        log_debug("CloudFront Distribution");

                        match find_cloudfront_distribution_id(&ctx.aws_client, dns_name).await {
                            Some(distribution_id) => {
                                insert_cloudfront_structure(ctx, dns_name, &distribution_id).await;
                            }
                            None => {
                                log_warn(format!(
                                    "TODO implement external CloudFront. No CloudFront distribution found with domain name '{}'. Location: main.rs: itterate_route53_records()",
                                    dns_name
                                ));
                            }
                        }
                    } else if dns_name.contains(".elb.") {
                        log_debug(format!("Elastic Load Balancer: {}", dns_name));
                        investigate_elb_by_hostname(
                            ctx,
                            hosted_zone_component_id,
                            dns_name,
                            display_key,
                        )
                        .await;
                    } else {
                        log_error("Unknown AWS Resource");
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
                let relation_addr = format!(
                    "{}.{}.{}.{}",
                    ctx.file_id, 0, 2, ctx.component_relation_count
                );
                let relation_id = convert_from_address_to_id(
                    relation_addr.clone(),
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
                log_warn(format!(
                    "No existing component found for alias target '{}', skipping component relation for record '{}'",
                    target_dns_name, record_name
                ));
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

/// Look up a CloudFront distribution id from its domain name (e.g. `XXX.cloudfront.net`).
#[tracing::instrument(skip(aws_client))]
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
                log_warn(format!(
                    "Listing CloudFront distributions failed: {}. Location: main.rs: find_cloudfront_distribution_id()",
                    err
                ));
                return None;
            }
        };
        if distribution.domain_name() == domain_name {
            return Some(distribution.id().to_string());
        }
    }

    None
}

/// Turn an expired SSO session or missing credentials error into an actionable message.
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

/// Index all load balancers by lowercased DNS name to avoid one API request per origin.
#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip(ctx))]
async fn investigate_elb_by_hostname(
    ctx: &mut AppContext,
    origin_component_id: u64,
    hostname: &str,
    display_key: &str,
) {
    let elb_arn = match ctx
        .elb_dns_name_to_load_balancer
        .get(&hostname.to_lowercase())
    {
        Some(load_balancer) => load_balancer
            .load_balancer_arn()
            .unwrap_or(hostname)
            .to_string(),
        None => {
            log_warn(format!(
                "No load balancer found with DNS name '{}'. Location: main.rs: investigate_elb_by_hostname()",
                hostname
            ));
            hostname.to_string()
        }
    };

    investigate_elb(ctx, origin_component_id, hostname, &elb_arn, display_key).await;
}

/// Kubernetes resource type that owns an ELB, based on its stack tag.
/// `service.k8s.aws/stack` is `namespace/name`. `ingress.k8s.aws/stack` is a bare group name
/// when the ALB is shared by an ingress group.
#[derive(Clone, Copy)]
enum K8sBackendKind {
    Gateway,
    Ingress,
}

/// Add an ELB component related to the origin component, and follow its tags to the owning
/// Kubernetes Gateway or Ingress.
#[tracing::instrument(skip(ctx))]
async fn investigate_elb(
    ctx: &mut AppContext,
    origin_component_id: u64,
    elb_hostname: &str,
    elb_arn: &str,
    display_key: &str,
) {
    if let Some(&elb_component_id) = ctx.elb_hostname_to_component_id.get(elb_hostname) {
        // TODO: Print elb_component_id in dotted address format.
        log_debug(format!(
            "ELB '{}' already investigated, reusing existing component. Location: main.rs: investigate_elb()",
            elb_arn
        ));
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

    log_debug(format!("elb_arn: {}", elb_arn));

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
                    log_warn(format!(
                        "No load balancer found for ARN '{}'. Location: main.rs: investigate_elb()",
                        elb_arn
                    ));
                }
            }
            Err(err) => {
                log_warn(format!(
                    "Describing load balancer '{}' failed: {}. Location: main.rs: investigate_elb()",
                    elb_arn, err
                ));
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
                    match tag.key() {
                        Some("ingress.k8s.aws/stack") => {
                            k8s_stack = tag.value().unwrap_or("").to_string();
                            k8s_backend_kind = Some(K8sBackendKind::Ingress);
                            log_debug(format!("ingress.k8s.aws/stack: {}", k8s_stack))
                        }
                        Some("service.k8s.aws/stack") => {
                            k8s_stack = tag.value().unwrap_or("").to_string();
                            k8s_backend_kind = Some(K8sBackendKind::Gateway);
                            log_debug(format!("service.k8s.aws/stack: {}", k8s_stack))
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
                        _ => {}
                    }
                }
            }
            Err(err) => {
                log_warn(format!(
                    "Describing tags for '{}' failed: {}. Location: main.rs: investigate_elb()",
                    elb_arn, err
                ));
            }
        }
        if k8s_stack.is_empty() {
            log_warn(format!("k8s_stack is empty for elb_arn: {}", elb_arn))
        }
    } else {
        log_error(format!("elb_arn not an ARN: {}", elb_arn));
    }

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

    let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
    let elb_component_id = convert_from_address_to_id(component_addr, "main.rs: investigate_elb()");
    let title_name = if elb_name.is_empty() {
        elb_arn.rsplit('/').next().unwrap_or(elb_arn).to_string()
    } else {
        elb_name
    };
    let component_title = format!("ELB {}", title_name);
    insert_into_component(
        &ctx.db_conn,
        ctx.file_id,
        elb_component_id,
        &elb_hostname,
        "",
        &summary,
        ctx.team_id,
        &component_title,
    );
    ctx.component_count += 1;
    ctx.elb_hostname_to_component_id
        .insert(elb_hostname.to_string(), elb_component_id);

    log_debug(format!("investigate_k8s_ingresses for k8s_stack: {}", k8s_stack));

    let Some(backend_kind) = k8s_backend_kind else {
        log_warn("No ingress.k8s.aws/stack or service.k8s.aws/stack tag found, skipping Kubernetes ingress investigation. Location: main.rs: investigate_elb()");
        return;
    };

    let Some(k8s_client) = build_k8s_client(&ctx.eks_client, &k8s_cluster).await else {
        return;
    };

    match backend_kind {
        K8sBackendKind::Gateway => {
            log_debug(format!("ELB '{}' Looking for gateways", elb_arn));

            investigate_gateway_http_routes(
                ctx,
                elb_component_id,
                display_key,
                k8s_client,
                &k8s_stack,
            )
            .await;
        }
        K8sBackendKind::Ingress => {
            log_debug(format!("ELB '{}' Looking for ingress", elb_arn));
            investigate_ingress_backends(
                ctx,
                elb_component_id,
                display_key,
                k8s_client,
                &k8s_stack,
            )
            .await;
        }
    }
}

/// Add a Gateway and its HTTPRoute backends as components related to the ELB component.
/// `k8s-openapi` has no HTTPRoute type, so routes are read as `DynamicObject`.
#[tracing::instrument(skip(ctx, k8s_client))]
async fn investigate_gateway_http_routes(
    ctx: &mut AppContext,
    elb_component_id: u64,
    display_key: &str,
    k8s_client: K8sClient,
    k8s_stack: &str,
) {
    let (gateway_namespace, gateway_name) = k8s_stack.split_once("/").unwrap_or(("", k8s_stack));
    if gateway_namespace.is_empty() {
        // TODO: Panic here, since every Gateway API stack tag is `namespace/name`.
        log_error(format!("k8s_stack could not be split on '/': {}", k8s_stack))
    };

    // Several ELBs can front the same Gateway, so the Gateway component is keyed by its unique
    // name and reused on a repeat hit.
    let gateway_component_name = format!("{}-Gateway", k8s_stack);
    let gateway_component_id =
        match get_component_by_name(&ctx.db_conn, gateway_component_name.clone()) {
            Ok(component) => component.id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
                let gateway_id = convert_from_address_to_id(
                    component_addr,
                    "main.rs: investigate_gateway_http_routes()",
                );
                let summary = format!("Gateway: {}", k8s_stack);
                let component_title = format!("Gateway {}", gateway_name);
                insert_into_component(
                    &ctx.db_conn,
                    ctx.file_id,
                    gateway_id,
                    &gateway_component_name,
                    "",
                    &summary,
                    ctx.team_id,
                    &component_title,
                );
                ctx.component_count += 1;

                // TODO: The relation is only created with the Gateway component, so a second ELB
                // fronting the same Gateway gets no relation. Check whether that case exists.
                let relation_addr = format!(
                    "{}.{}.{}.{}",
                    ctx.file_id, 0, 2, ctx.component_relation_count
                );
                let relation_id = convert_from_address_to_id(
                    relation_addr.clone(),
                    "main.rs: investigate_gateway_http_routes()",
                );
                log_debug(format!(
                    "Adding component relation {} between ELB and Gateway '{}'",
                    relation_addr, k8s_stack
                ));
                insert_into_component_relation(
                    &ctx.db_conn,
                    relation_id,
                    0,
                    elb_component_id,
                    gateway_id,
                    "connect",
                    display_key,
                    "",
                    "",
                    "",
                    "",
                );
                ctx.component_relation_count += 1;

                gateway_id
            }
            Err(err) => panic!("expected a component, got an error: {err}"),
        };

    let http_route_resource = ApiResource::from_gvk_with_plural(
        &GroupVersionKind::gvk("gateway.networking.k8s.io", "v1", "HTTPRoute"),
        "httproutes",
    );
    let http_routes: Api<DynamicObject> = Api::all_with(k8s_client, &http_route_resource);
    let http_route_list = match http_routes.list(&ListParams::default()).await {
        Ok(list) => list,
        Err(err) => {
            log_error(format!(
                "Listing HTTPRoutes for stack '{}' failed: {}. Location: main.rs: investigate_gateway_http_routes()",
                k8s_stack, err
            ));
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

        if ctx.filtered_namespace.is_empty() || meata_namespace == ctx.filtered_namespace {
            log_debug(format!(
                "httproute meta_ns: {} meta_name: {}",
                meata_namespace, meta_name
            ));
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

            // A parentRef without a namespace refers to the HTTPRoute's own namespace.
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

            let correct_ns_and_container = if let Some((parent_namespace, parent_container_name)) =
                parent_refs.first()
            {
                // The stack tag names the Service in front of the Gateway, which NGINX Gateway
                // Fabric names `<gateway>-nginx`.
                let parent_full_container_name = format!("{}-nginx", parent_container_name);
                if parent_full_container_name == gateway_name
                    && *parent_namespace == gateway_namespace
                {
                    log_debug("NS and container names fit");
                    true
                } else {
                    log_debug(format!(
                        "NS and container names doe NOT fit. parent NS {}, parent_container name: {}",
                        parent_namespace, parent_full_container_name
                    ));
                    false
                }
            } else {
                false
            };

            if correct_ns_and_container {
                log_debug("Correct HTTPRoute found.");

                // A rule can have several backendRefs. A backendRef without a kind is a Service,
                // per the Gateway API spec.
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

                log_debug(format!(
                    "httproute {} backend_refs: {:?}",
                    meta_name, backend_refs
                ));
                if backend_refs.len() > 1 {
                    log_warn(format!(
                        "backendRefs has more than one entry, actual: {}",
                        backend_refs.len()
                    ));
                }
                if let Some((backend_kind, backend_name)) = backend_refs.first() {
                    if *backend_kind != "Service" {
                        log_error(format!(
                            "unexpected backendRefs kind(expected Service) found: {} for name: {}",
                            backend_kind, backend_name
                        ));
                    } else {
                        record_backend_relation(
                            ctx,
                            gateway_component_id,
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

/// Add the Ingresses of an ALB ingress group and their Service backends as components
/// related to the ELB component.
#[tracing::instrument(skip(ctx, k8s_client))]
async fn investigate_ingress_backends(
    ctx: &mut AppContext,
    elb_component_id: u64,
    display_key: &str,
    k8s_client: K8sClient,
    ingress_group_name: &str,
) {
    let mut found_ingress_for_groupname = false;

    let ingresses: Api<Ingress> = Api::all(k8s_client);
    let ingress_list = match ingresses.list(&ListParams::default()).await {
        Ok(list) => list,
        Err(err) => {
            log_error(format!(
                "Listing Ingresses failed: {}. Location: main.rs: investigate_ingress_backends()",
                err
            ));
            return;
        }
    };

    for ingress in ingress_list {
        let ingress_namespace = ingress.metadata.namespace.as_deref().unwrap_or("default");
        let ingress_name = ingress.metadata.name.as_deref().unwrap_or("Unknown");

        if !ctx.filtered_namespace.is_empty() && ingress_namespace != ctx.filtered_namespace {
            continue;
        }

        if !ingress
            .metadata
            .annotations
            .as_ref()
            .and_then(|annotations| annotations.get("alb.ingress.kubernetes.io/group.name"))
            .is_some_and(|group_name| group_name == ingress_group_name)
        {
            continue;
        }

        found_ingress_for_groupname = true;

        // Keyed by namespace/name so a repeat hit reuses the existing Ingress component.
        let ingress_component_name = format!("{}/{}-Ingress", ingress_namespace, ingress_name);
        let ingress_component_id =
            match get_component_by_name(&ctx.db_conn, ingress_component_name.clone()) {
                Ok(component) => component.id,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    let component_addr = format!("{}.0.0.{}", ctx.file_id, ctx.component_count);
                    let ingress_id = convert_from_address_to_id(
                        component_addr,
                        "main.rs: investigate_ingress_backends()",
                    );
                    let summary = format!("Ingress: {}/{}", ingress_namespace, ingress_name);
                    let component_title = format!("Ingress {}", ingress_name);
                    insert_into_component(
                        &ctx.db_conn,
                        ctx.file_id,
                        ingress_id,
                        &ingress_component_name,
                        "",
                        &summary,
                        ctx.team_id,
                        &component_title,
                    );
                    ctx.component_count += 1;

                    let relation_addr = format!(
                        "{}.{}.{}.{}",
                        ctx.file_id, 0, 2, ctx.component_relation_count
                    );
                    let relation_id = convert_from_address_to_id(
                        relation_addr.clone(),
                        "main.rs: investigate_ingress_backends()",
                    );
                    log_debug(format!(
                        "Adding component relation {} between ELB and Ingress '{}/{}'",
                        relation_addr, ingress_namespace, ingress_name
                    ));
                    insert_into_component_relation(
                        &ctx.db_conn,
                        relation_id,
                        0,
                        elb_component_id,
                        ingress_id,
                        "connect",
                        display_key,
                        "",
                        "",
                        "",
                        "",
                    );
                    ctx.component_relation_count += 1;

                    ingress_id
                }
                Err(err) => panic!("expected a component, got an error: {err}"),
            };

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
                            ingress_component_id,
                            display_key,
                            "Service",
                            &service_backend.name,
                            &[host],
                        );
                    }
                    None => {
                        log_error(format!(
                            "Ingress {}/{} path backend has no Service (a custom resource backend?), skipping. Location: main.rs: investigate_ingress_backends()",
                            ingress_namespace, ingress_name
                        ));
                    }
                }
            }
        }
    }

    if !found_ingress_for_groupname {
        log_warn(format!(
            "No Ingress found for the ingress group name: '{}'. Location: main.rs: investigate_ingress_backends()",
            ingress_group_name
        ));
    }
}

/// Get or create a backend component and relate it to the upstream component once per hostname.
#[tracing::instrument(skip(ctx))]
fn record_backend_relation(
    ctx: &mut AppContext,
    parrent_component_id: u64,
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
            // TODO: Determine the component type of the backend.
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
        let relation_addr = format!(
            "{}.{}.{}.{}",
            ctx.file_id, 0, 2, ctx.component_relation_count
        );
        let relation_id =
            convert_from_address_to_id(relation_addr.clone(), "main.rs: record_backend_relation()");
        log_debug(format!(
            "Adding component relation for component id: {} hostname: '{}'",
            relation_addr, hostname
        ));
        insert_into_component_relation(
            &ctx.db_conn,
            relation_id,
            0,
            parrent_component_id,
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

/// Build a `kube::Client` for an EKS cluster. Returns `None` with a warning on any failure.
#[tracing::instrument(skip(eks_client))]
async fn build_k8s_client(eks_client: &EksClient, k8s_cluster_name: &str) -> Option<K8sClient> {
    if k8s_cluster_name.is_empty() {
        log_debug("EKS cluster name not defined, skipped build_k8s_client");
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
                log_warn(format!(
                    "EKS cluster '{}' not found. Location: main.rs: build_k8s_client()",
                    k8s_cluster_name
                ));
                return None;
            }
        },
        Err(err) => {
            log_warn(format!(
                "Describing EKS cluster '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            ));
            return None;
        }
    };

    let Some(endpoint) = cluster.endpoint() else {
        log_warn(format!(
            "EKS cluster '{}' has no API endpoint. Location: main.rs: build_k8s_client()",
            k8s_cluster_name
        ));
        return None;
    };
    let cluster_url: http::Uri = match endpoint.parse() {
        Ok(url) => url,
        Err(err) => {
            log_warn(format!(
                "Parsing endpoint '{}' for cluster '{}' failed: {}. Location: main.rs: build_k8s_client()",
                endpoint, k8s_cluster_name, err
            ));
            return None;
        }
    };

    let Some(ca_data) = cluster.certificate_authority().and_then(|ca| ca.data()) else {
        log_warn(format!(
            "EKS cluster '{}' has no certificate authority data. Location: main.rs: build_k8s_client()",
            k8s_cluster_name
        ));
        return None;
    };
    let ca_pem = match base64::engine::general_purpose::STANDARD.decode(ca_data) {
        Ok(pem) => pem,
        Err(err) => {
            log_warn(format!(
                "Decoding certificate authority data for '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            ));
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
            log_warn(format!(
                "Parsing certificate authority data for '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            ));
            return None;
        }
    };

    let bearer_token = match get_eks_bearer_token(k8s_cluster_name) {
        Ok(token) => token,
        Err(err) => {
            log_warn(format!(
                "Getting a bearer token for EKS cluster '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            ));
            return None;
        }
    };

    let mut k8s_config = K8sConfig::new(cluster_url);
    k8s_config.root_cert = Some(root_cert);
    k8s_config.auth_info.token = Some(SecretString::from(bearer_token));
    // `kube` has no default timeouts, so an unreachable private endpoint hangs forever.
    k8s_config.connect_timeout = Some(std::time::Duration::from_secs(10));
    k8s_config.read_timeout = Some(std::time::Duration::from_secs(30));

    match K8sClient::try_from(k8s_config) {
        Ok(client) => Some(client),
        Err(err) => {
            log_warn(format!(
                "Building a Kubernetes client for '{}' failed: {}. Location: main.rs: build_k8s_client()",
                k8s_cluster_name, err
            ));
            None
        }
    }
}

/// Get an EKS bearer token via `aws eks get-token`. No AWS API returns one directly.
#[tracing::instrument(err)]
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
