#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ActionError {
    CannotPassThroughWall,
    CannotPassThroughOpponent,
    NoRunningChallenge,
    SolveChallengeFirst,
    InvalidChallengeSolution,
}