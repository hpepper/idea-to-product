// TODO do I need sort_order and view_packet_id I do not think so
#[derive(Debug)]
pub struct Behavior {
    pub id: i32,
    pub sort_order: i32,
    pub view_packet_id: i32,
    pub description: String,
    pub diagram_key: String,
}

#[derive(Debug)]
pub struct Component {
    pub id: i32,
    pub name: String,
    pub summary: String,
    pub purpose: String,
    pub team_id: i32,
}

// TODO Add Id from attribute
#[derive(Debug)]
pub struct ComponentRelation {
    pub id: i32,
    pub component_a_id: i32,
    pub component_b_id: i32,
    pub connection_type: String,
    pub key: String,
    pub property_of_relation: String,
    pub relation_text: String,
    pub relation_description: String,
    pub sort_order: i32,
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
pub struct ViewPacket {
    pub component_id: i32,
    pub context_model_key: String,
    pub primary_display_key: String,
    pub introduction: String,
    pub sort_order: i32,
    pub team_id: i32,
    pub title: String,
    pub view_style: String,
    pub view_type: String,
    pub viewpacket_id: i32,
}

#[derive(Debug)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug)]
pub struct TeamMember {
    pub team_id: i32,
    pub member_name: String,
    pub role: String,
}
