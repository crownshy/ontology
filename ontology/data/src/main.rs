use std::collections::HashMap;

use schemars::{schema_for, JsonSchema};
use uuid::Uuid;

#[derive(JsonSchema)]
/// The role that a given Statement plays 
/// This might be determined by the 
/// creator of the Statement, some algorithmic 
/// method or assigned by a host 
pub enum StatementRole{
    /// The statement represents a question being possed
    Question,
    /// The statement represents a Belief
    Belief,
    /// The statement represents a Fact
    Fact,
    /// The statement represents a Evidence provided in support 
    /// of another Statement
    Evidence,
    /// The statement refutes or disagrees with another  
    Refutation,
    /// The statement is a summary of another process (for example 
    /// a Grouping or consensus)
    Summary,
    /// A proposal that has been made    
    Proposal, 
    /// A decision that has been made
    Decision
}

#[derive(JsonSchema)]
/// A way to describe an algorithmic process that 
/// has been used to generate other entities in the 
/// schema.
pub struct AlgorithmDescriptor{
    pub id: Uuid,
    /// The type of algorithm being used
    pub kind: String,
    /// Metadata about how the algorithm is setup
    pub metadata: HashMap<String,String>
}

#[derive(JsonSchema)]
/// Details of who generated an entity in the schema 
pub enum Generator{
    /// A participant 
    Participant(Uuid),
    /// A host of the process
    Host(Uuid),
    /// A algorithmic process 
    Algorithm(AlgorithmDescriptor)
}

#[derive(JsonSchema)]
/// A statement which has been made by some entity in the system 
pub struct Statement{
    pub id: Uuid,
    /// The content of the statement
    pub content: String,
    /// The role the statement plays
    pub role: StatementRole,
    /// The person, entity or algorithm that generated the statement
    pub made_by: Generator,
    /// If this statement was made in response to another 
    pub in_response_to: Option<Uuid>, 
    /// How was the role of this statement generated
    pub role_classified_by: Generator
}

#[derive(JsonSchema)]
/// A vote in response to another entity on the system
pub enum SimpleVote{
   /// The generator agreed
   Positive,
   /// The generator disagreed
   Negative,
   /// The generator was neutral 
   Neutral
} 

#[derive(JsonSchema)]
/// A quadratic vote registered in response to the entity 
pub struct QuadVote{
    /// What level did the generator vote with 
    pub amount: i32,
    /// What was the cost of the vote
    pub cost: f32 
}

#[derive(JsonSchema)]
/// A generic score that a generator assigned to an entity
pub struct Score(pub i32);

#[derive(JsonSchema)]
/// The type of reaction to a given entity
pub enum ReactionType {
   SimpleVote(SimpleVote),
   Score(Score),
   QuadraticVote(QuadVote),
}

#[derive(JsonSchema)]
/// A reaction to an entity on the system
pub struct Reaction{
    pub id:Uuid,
    /// Who is making this reaction
    pub made_by:Generator,
    /// The reaction itself
    pub reaction: ReactionType,
    /// The type of entity being reacted to 
    pub entity_type: EntityType
}

#[derive(JsonSchema)]
/// A type of entity on the system
pub enum EntityType{
    /// A participant in the conversation
    Participant,
    /// A Statement made by the system
    Statement,
    /// A Group on the system 
    Group,
    /// A reaction 
    Reaction
}

#[derive(JsonSchema)]
/// Represents a grouping, created either manually or algorithmically 
/// of entities.
pub struct Group{
    pub id: Uuid,
    /// The type of thing being grouped 
    pub entity_type: EntityType,
    /// A list of the entity Uuids in this group
    pub entities: Vec<Uuid>, 
    /// Who generated this group
    pub made_by: Generator
}


#[derive(JsonSchema)]
struct AllTypes{
    pub group:Group,
    pub statement: Statement,
    pub reaction: Reaction, 
}

fn main() {
    let statement_schema = schema_for!(AllTypes);
    println!("{}", serde_json::to_string_pretty(&statement_schema).unwrap());
}
