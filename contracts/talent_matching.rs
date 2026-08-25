// AI-Powered Talent Matching - Issue #583

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TalentProfile {
    pub id: String,
    pub name: String,
    pub skills: Vec<String>,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct ProjectRequirement {
    pub project_id: String,
    pub required_skills: Vec<String>,
    pub min_score: f64,
}

#[derive(Debug, Default)]
pub struct TalentMatcher {
    pub profiles: HashMap<String, TalentProfile>,
}

impl TalentMatcher {
    pub fn new() -> Self {
        Self { profiles: HashMap::new() }
    }

    pub fn register_talent(&mut self, id: String, name: String, skills: Vec<String>, score: f64) {
        self.profiles.insert(id.clone(), TalentProfile { id, name, skills, score });
    }

    pub fn find_matches(&self, req: &ProjectRequirement) -> Vec<&TalentProfile> {
        self.profiles.values()
            .filter(|p| {
                p.score >= req.min_score &&
                req.required_skills.iter().any(|s| p.skills.contains(s))
            })
            .collect()
    }

    pub fn get_top_match(&self, req: &ProjectRequirement) -> Option<&TalentProfile> {
        self.find_matches(req).into_iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }
}