
#[derive(Debug)]
pub struct ViewPacket {
    pub(crate) component_id: i32,
    pub(crate) context_model_key: String,
    pub(crate) primary_display_key: String,
    pub(crate) introduction: String,
    pub(crate) sort_order: i32,
    pub(crate) title: String,
    pub(crate) view_style: String,
    pub(crate) view_type: String,
    pub(crate) viewpacket_id: i32,
}

// TODO Add Id from attribute
#[derive(Debug)]
pub struct ComponentRelation {
    pub(crate) id: i32,
    pub(crate) component_a_id: i32,
    pub(crate) component_b_id: i32,
    pub(crate) connection_type: String,
    pub(crate) key: String,
    pub(crate) property_of_relation: String,
    pub(crate) relation_text: String,
}

#[derive(Debug)]
pub struct Component {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) summary: String,
}

// TODO do I need sort_order and view_packet_id I do not think so
#[derive(Debug)]
pub struct Behavior {
    pub(crate) id: i32,
    pub(crate) sort_order: i32,
    pub(crate) view_packet_id: i32,
    pub(crate) description: String,
    pub(crate) diagram_key: String,
}
