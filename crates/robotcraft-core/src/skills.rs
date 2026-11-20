//! Bundled RobotCraft skills (`skills/*/SKILL.md`) embedded at compile time.

#[derive(Debug, Clone, Copy)]
pub struct BundledSkill {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub uri: &'static str,
    pub content: &'static str,
}

const SKILLS: &[BundledSkill] = &[
    BundledSkill {
        id: "robotcraft-workflow",
        title: "RobotCraft Operator Workflow",
        summary: "Probe fleet peers, inspect live vs fixture telemetry, pose URDFs, and dispatch confirm-gated teleop.",
        uri: "robotcraft://skills/robotcraft-workflow",
        content: include_str!("../../../skills/robotcraft-workflow/SKILL.md"),
    },
    BundledSkill {
        id: "robotcraft-urdf-kinematics",
        title: "URDF & Kinematics",
        summary: "Nori URDF link posing, planar forward kinematics, and 0.10m-0.85m reachability sphere validation.",
        uri: "robotcraft://skills/robotcraft-urdf-kinematics",
        content: include_str!("../../../skills/robotcraft-urdf-kinematics/SKILL.md"),
    },
    BundledSkill {
        id: "robotcraft-safety",
        title: "Safety Gates & E-Stop",
        summary: "Yahboom motion confirm gates, VLA whitelist & X-VLA-Confirm, Isaac job gate, peer timeouts, and stop_all.",
        uri: "robotcraft://skills/robotcraft-safety",
        content: include_str!("../../../skills/robotcraft-safety/SKILL.md"),
    },
];

/// Return all bundled skills embedded in `robotcraft-core`.
pub fn bundled_skills() -> &'static [BundledSkill] {
    SKILLS
}

/// Look up a bundled skill by its short id (e.g. `"robotcraft-workflow"`) or resource URI.
pub fn get_skill(id_or_uri: &str) -> Option<&'static BundledSkill> {
    let key = id_or_uri.trim();
    SKILLS
        .iter()
        .find(|skill| skill.id == key || skill.uri == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_skills_are_embedded_and_resolvable() {
        let all = bundled_skills();
        assert_eq!(all.len(), 3);
        for skill in all {
            assert!(!skill.id.is_empty());
            assert!(!skill.content.trim().is_empty());
            assert!(skill.uri.starts_with("robotcraft://skills/"));
            assert!(get_skill(skill.id).is_some());
            assert!(get_skill(skill.uri).is_some());
        }
        assert!(get_skill("nonexistent-skill").is_none());
    }
}
