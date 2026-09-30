use std::collections::HashMap;

pub const MODULE_VIEW_TYPE: &str = "Module";
pub const CNC_VIEW_TYPE: &str = "CnC";
pub const ALLOCATION_VIEW_TYPE: &str = "Allocation";

pub const VIEW_TYPE_LIST: &[&str] = &[MODULE_VIEW_TYPE, CNC_VIEW_TYPE, ALLOCATION_VIEW_TYPE];

pub const MODULE_VIEW_TYPE_STYLE_DECOMPOSITION: &str = "Decomposition";
pub const MODULE_VIEW_TYPE_STYLE_USES: &str = "Uses";
pub const MODULE_VIEW_TYPE_STYLE_USEDBY: &str = "UsedBy";
pub const MODULE_VIEW_TYPE_STYLE_GENERALIZE: &str = "Generalize";
pub const MODULE_VIEW_TYPE_STYLE_LAYERED: &str = "Layered";

pub const CNC_VIEW_TYPE_STYLE_CLIENTSERVER: &str = "ClientServer";
pub const CNC_VIEW_TYPE_STYLE_PEERTOPEER: &str = "PeerToPeer";
pub const CNC_VIEW_TYPE_STYLE_PUBSUB: &str = "PublishSubscribe";
pub const CNC_VIEW_TYPE_STYLE_PIPEANDFILTER: &str = "PipeAndFilter";
pub const CNC_VIEW_TYPE_STYLE_SHAREDDATA: &str = "SharedData";

pub const ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMENT: &str = "Deployment";
pub const ALLOCATION_VIEW_TYPE_STYLE_INSTALL: &str = "Install";
pub const ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT: &str = "Assignment";
pub const ALLOCATION_VIEW_TYPE_STYLE_TESTING: &str = "Testing";

pub const CONNECTION_TYPE_DEPLOYMENT_CONTAINS: &str = "Contains";
pub const CONNECTION_TYPE_DEPLOYMENT_CONNECT: &str = "Connect";
pub const CONNECTION_TYPE_DEPLOYMENT_ALLIGN: &str = "Allign";

pub fn create_hardcoded_map() -> HashMap<&'static str, u64> {
    let mut map = HashMap::new();
    map.insert(MODULE_VIEW_TYPE, 1);
    map.insert(CNC_VIEW_TYPE, 2);
    map.insert(ALLOCATION_VIEW_TYPE, 3);
    map.insert(MODULE_VIEW_TYPE_STYLE_DECOMPOSITION, 1);
    map.insert(MODULE_VIEW_TYPE_STYLE_USES, 2);
    map.insert(MODULE_VIEW_TYPE_STYLE_USEDBY, 3);
    map.insert(MODULE_VIEW_TYPE_STYLE_GENERALIZE, 4);
    map.insert(MODULE_VIEW_TYPE_STYLE_LAYERED, 5);
    map.insert(CNC_VIEW_TYPE_STYLE_CLIENTSERVER, 1);
    map.insert(CNC_VIEW_TYPE_STYLE_PEERTOPEER, 2);
    map.insert(CNC_VIEW_TYPE_STYLE_PUBSUB, 3);
    map.insert(CNC_VIEW_TYPE_STYLE_PIPEANDFILTER, 4);
    map.insert(CNC_VIEW_TYPE_STYLE_SHAREDDATA, 5);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMENT, 1);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_INSTALL, 2);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT, 3);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_TESTING, 4);

    map
}

// Module: Decomposition, Uses, Generalize, Layered
// CnC: ClientServer, PeerToPeer, PublishSubscribe, PeerToPeer, PipeAndFilter, PublishSubscribe, SharedData
// Allocation: Deployment, Implemnetation, Assignment
pub fn create_styles() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();
    map.insert(
        MODULE_VIEW_TYPE,
        vec![
            MODULE_VIEW_TYPE_STYLE_DECOMPOSITION,
            MODULE_VIEW_TYPE_STYLE_USES,
            MODULE_VIEW_TYPE_STYLE_USEDBY,
            MODULE_VIEW_TYPE_STYLE_GENERALIZE,
            MODULE_VIEW_TYPE_STYLE_LAYERED,
        ],
    );
    map.insert(
        CNC_VIEW_TYPE,
        vec![
            CNC_VIEW_TYPE_STYLE_CLIENTSERVER,
            CNC_VIEW_TYPE_STYLE_PEERTOPEER,
            CNC_VIEW_TYPE_STYLE_PUBSUB,
            CNC_VIEW_TYPE_STYLE_PIPEANDFILTER,
            CNC_VIEW_TYPE_STYLE_SHAREDDATA,
        ],
    );
    map.insert(
        ALLOCATION_VIEW_TYPE,
        vec![
            ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMENT,
            ALLOCATION_VIEW_TYPE_STYLE_INSTALL,
            ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT,
            ALLOCATION_VIEW_TYPE_STYLE_TESTING,
        ],
    );
    map
}

/// Convert an address of the form "file_id.a.b.c" into a numerical ID.
pub fn convert_from_address_to_id(addr: String, location: &str) -> u64 {
    // TODO: Implement the conversion logic
    // TODO split "a.b.c" into parts and calculate the id.
    let parts: Vec<&str> = addr.split('.').collect();
    if parts.len() == 4 {
        let alpha: u64 = parts[0].parse().unwrap_or(0);
        let bravo: u64 = parts[1].parse().unwrap_or(0);
        let charlie: u64 = parts[2].parse().unwrap_or(0);
        let delta: u64 = parts[3].parse().unwrap_or(0);
        return alpha * 256 * 256 * 256 + bravo * 256 * 256 + charlie * 256 + delta;
    } else {
        panic!(
            "!!! Address '{}' is not in the correct format 'a.b.c.d'. Location: {}",
            addr, location
        );
    }
}

/// Convert a numerical ID into an address of the form "file_id.a.b.c".
pub fn convert_from_id_to_address(id: u64) -> String {
    let alpha = id / (256 * 256 * 256);
    let mut temp_id = id - (alpha * 256 * 256 * 256);
    let bravo = temp_id / (256 * 256);
    temp_id = temp_id - (bravo * 256 * 256);
    let charlie = temp_id / 256;
    let delta = temp_id - (charlie * 256);
    format!("{}.{}.{}.{}", alpha, bravo, charlie, delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hardcoded_map_contains_all_view_types() {
        let map = create_hardcoded_map();
        assert_eq!(map[MODULE_VIEW_TYPE], 1);
        assert_eq!(map[CNC_VIEW_TYPE], 2);
        assert_eq!(map[ALLOCATION_VIEW_TYPE], 3);
    }

    #[test]
    fn test_create_hardcoded_map_contains_all_module_styles() {
        let map = create_hardcoded_map();
        assert_eq!(map[MODULE_VIEW_TYPE_STYLE_DECOMPOSITION], 1);
        assert_eq!(map[MODULE_VIEW_TYPE_STYLE_USES], 2);
        assert_eq!(map[MODULE_VIEW_TYPE_STYLE_USEDBY], 3);
        assert_eq!(map[MODULE_VIEW_TYPE_STYLE_GENERALIZE], 4);
        assert_eq!(map[MODULE_VIEW_TYPE_STYLE_LAYERED], 5);
    }

    #[test]
    fn test_create_hardcoded_map_contains_all_cnc_styles() {
        let map = create_hardcoded_map();
        assert_eq!(map[CNC_VIEW_TYPE_STYLE_CLIENTSERVER], 1);
        assert_eq!(map[CNC_VIEW_TYPE_STYLE_PEERTOPEER], 2);
        assert_eq!(map[CNC_VIEW_TYPE_STYLE_PUBSUB], 3);
        assert_eq!(map[CNC_VIEW_TYPE_STYLE_PIPEANDFILTER], 4);
        assert_eq!(map[CNC_VIEW_TYPE_STYLE_SHAREDDATA], 5);
    }

    #[test]
    fn test_create_hardcoded_map_contains_all_allocation_styles() {
        let map = create_hardcoded_map();
        assert_eq!(map[ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMENT], 1);
        assert_eq!(map[ALLOCATION_VIEW_TYPE_STYLE_INSTALL], 2);
        assert_eq!(map[ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT], 3);
        assert_eq!(map[ALLOCATION_VIEW_TYPE_STYLE_TESTING], 4);
    }

    #[test]
    fn test_create_hardcoded_map_size() {
        let map = create_hardcoded_map();
        // 3 view types + 5 module styles + 5 CnC styles + 4 allocation styles
        assert_eq!(map.len(), 17);
    }

    #[test]
    fn test_view_type_list_contains_all_view_types() {
        assert_eq!(
            VIEW_TYPE_LIST,
            &[MODULE_VIEW_TYPE, CNC_VIEW_TYPE, ALLOCATION_VIEW_TYPE]
        );
    }

    #[test]
    fn test_create_styles_contains_all_view_types() {
        let styles = create_styles();
        assert!(styles.contains_key(MODULE_VIEW_TYPE));
        assert!(styles.contains_key(CNC_VIEW_TYPE));
        assert!(styles.contains_key(ALLOCATION_VIEW_TYPE));
        assert_eq!(styles.len(), 3);
    }

    #[test]
    fn test_create_styles_module_styles() {
        let styles = create_styles();
        assert_eq!(
            styles[MODULE_VIEW_TYPE],
            vec![
                MODULE_VIEW_TYPE_STYLE_DECOMPOSITION,
                MODULE_VIEW_TYPE_STYLE_USES,
                MODULE_VIEW_TYPE_STYLE_USEDBY,
                MODULE_VIEW_TYPE_STYLE_GENERALIZE,
                MODULE_VIEW_TYPE_STYLE_LAYERED,
            ]
        );
    }

    #[test]
    fn test_create_styles_cnc_styles() {
        let styles = create_styles();
        assert_eq!(
            styles[CNC_VIEW_TYPE],
            vec![
                CNC_VIEW_TYPE_STYLE_CLIENTSERVER,
                CNC_VIEW_TYPE_STYLE_PEERTOPEER,
                CNC_VIEW_TYPE_STYLE_PUBSUB,
                CNC_VIEW_TYPE_STYLE_PIPEANDFILTER,
                CNC_VIEW_TYPE_STYLE_SHAREDDATA,
            ]
        );
    }

    #[test]
    fn test_create_styles_allocation_styles() {
        let styles = create_styles();
        assert_eq!(
            styles[ALLOCATION_VIEW_TYPE],
            vec![
                ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMENT,
                ALLOCATION_VIEW_TYPE_STYLE_INSTALL,
                ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT,
                ALLOCATION_VIEW_TYPE_STYLE_TESTING,
            ]
        );
    }

    #[test]
    fn test_convert_from_address_to_id() {
        let id = convert_from_address_to_id("1.2.3.4".to_string(), "test");
        assert_eq!(id, 16909060);
    }

    #[test]
    fn test_convert_from_address_to_id_zero() {
        let id = convert_from_address_to_id("0.0.0.0".to_string(), "test");
        assert_eq!(id, 0);
    }

    #[test]
    fn test_convert_from_address_to_id_non_numeric_parts_default_to_zero() {
        let id = convert_from_address_to_id("a.b.c.d".to_string(), "test");
        assert_eq!(id, 0);
    }

    #[test]
    #[should_panic(expected = "is not in the correct format")]
    fn test_convert_from_address_to_id_too_few_parts_panics() {
        convert_from_address_to_id("1.2.3".to_string(), "test");
    }

    #[test]
    #[should_panic(expected = "is not in the correct format")]
    fn test_convert_from_address_to_id_too_many_parts_panics() {
        convert_from_address_to_id("1.2.3.4.5".to_string(), "test");
    }

    #[test]
    fn test_convert_from_id_to_address() {
        let address = convert_from_id_to_address(16909060);
        assert_eq!(address, "1.2.3.4");
    }

    #[test]
    fn test_convert_from_id_to_address_zero() {
        let address = convert_from_id_to_address(0);
        assert_eq!(address, "0.0.0.0");
    }

    #[test]
    fn test_convert_from_id_to_address_roundtrip() {
        let id = convert_from_address_to_id("10.20.30.40".to_string(), "test");
        let address = convert_from_id_to_address(id);
        assert_eq!(address, "10.20.30.40");
    }
}
