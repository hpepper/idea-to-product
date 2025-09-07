// TODO do I need sort_order and view_packet_id I do not think so
#[derive(Debug)]
pub struct Behavior {
    pub id: u64,
    pub sort_order: u64,
    pub view_packet_id: u64,
    pub description: String,
    pub diagram_key: String,
}

#[derive(Debug)]
pub struct Component {
    pub id: u64,
    pub name: String,
    pub summary: String,
    pub purpose: String,
    pub team_id: u64,
}

// TODO Add Id from attribute
#[derive(Debug)]
pub struct ComponentRelation {
    pub id: u64,
    pub component_a_id: u64,
    pub component_b_id: u64,
    pub connection_type: String,
    pub key: String,
    pub property_of_relation: String,
    pub relation_text: String,
    pub relation_description: String,
    pub sort_order: u64,
    pub style: String,
}

#[derive(Debug)]
pub struct ContextModel {
    pub entity: String,
    pub entity_type: String,
    pub description: String,
    pub reference: String,
}

#[derive(Debug)]
pub struct Document  {
    pub file_id: u64,
    pub filename: String,
    pub title: String,
    pub issue: String,
    pub summary: String,
}

#[derive(Debug)]
pub struct Include  {
    pub file_id: u64,
    pub filename: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Team {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[derive(Debug)]
pub struct TeamMember {
    pub team_id: u64,
    pub member_name: String,
    pub role: String,
}

#[derive(Debug)]
pub struct ViewPacket {
    pub component_id: u64,
    pub context_model_key: String,
    pub primary_display_key: String,
    pub introduction: String,
    pub sort_order: u64,
    pub team_id: u64,
    pub title: String,
    pub view_style: String,
    pub view_type: String,
    pub viewpacket_id: u64,
}

