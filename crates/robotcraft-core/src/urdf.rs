//! URDF loader and posed link frames.
//!
//! Reads a URDF file, keeps revolute / continuous / prismatic joints, and
//! places each link in the base frame. Mesh files are not loaded. A link
//! with only a mesh visual falls back to its collision box, then to a
//! 4 cm cube at the link origin.

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct UrdfJoint {
    pub name: String,
    pub joint_type: String,
    pub parent: String,
    pub child: String,
    pub origin_xyz: [f32; 3],
    pub origin_rpy: [f32; 3],
    pub axis: [f32; 3],
    pub lower: f32,
    pub upper: f32,
}

#[derive(Debug, Clone)]
pub struct UrdfLink {
    pub name: String,
    pub size: [f32; 3],
    pub visual_origin: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct ParsedUrdf {
    pub name: String,
    pub links: Vec<UrdfLink>,
    pub joints: Vec<UrdfJoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkPose {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MovableJoint {
    pub name: String,
    pub joint_type: String,
    pub lower: f32,
    pub upper: f32,
    pub value: f32,
}

struct Xform {
    r: [[f32; 3]; 3],
    t: [f32; 3],
}

pub fn default_nori_urdf() -> PathBuf {
    match std::env::var("ROBOTCRAFT_NORI_URDF") {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value.trim()),
        _ => PathBuf::from(
            r"D:\Dev\repos\norirobotics-mcp\models\nori_description\urdf\nori.expanded.urdf",
        ),
    }
}

pub fn load_urdf_file(path: &Path) -> Result<ParsedUrdf, String> {
    let text =
        std::fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    parse_urdf(&text)
}

pub fn parse_urdf(xml: &str) -> Result<ParsedUrdf, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|err| format!("urdf xml: {err}"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "robot" {
        return Err("root element is not <robot>".to_string());
    }
    let name = root.attribute("name").unwrap_or("robot").to_string();
    let mut links = Vec::new();
    let mut joints = Vec::new();
    for node in root.children().filter(|n| n.is_element()) {
        match node.tag_name().name() {
            "link" => {
                if let Some(link_name) = node.attribute("name") {
                    links.push(parse_link(link_name, node));
                }
            }
            "joint" => {
                if let Some(joint) = parse_joint(node) {
                    joints.push(joint);
                }
            }
            _ => {}
        }
    }
    if links.is_empty() {
        return Err("urdf has no links".to_string());
    }
    Ok(ParsedUrdf {
        name,
        links,
        joints,
    })
}

pub fn pose_links(model: &ParsedUrdf, angles: &[(String, f32)]) -> Vec<LinkPose> {
    let root = root_link(model);
    let mut out = Vec::new();
    let mut guard = 0usize;
    walk(
        model,
        &root,
        &Xform::identity(),
        angles,
        &mut out,
        &mut guard,
    );
    out
}

pub fn movable_joints(model: &ParsedUrdf, angles: &[(String, f32)]) -> Vec<MovableJoint> {
    model
        .joints
        .iter()
        .filter(|joint| joint.joint_type != "fixed")
        .map(|joint| {
            let value = angles
                .iter()
                .find(|(name, _)| name == &joint.name)
                .map(|(_, value)| *value)
                .unwrap_or(0.0);
            let (lower, upper) = slider_limits(joint);
            MovableJoint {
                name: joint.name.clone(),
                joint_type: joint.joint_type.clone(),
                lower,
                upper,
                value: value.clamp(lower, upper),
            }
        })
        .collect()
}

fn slider_limits(joint: &UrdfJoint) -> (f32, f32) {
    if joint.joint_type == "continuous" || joint.upper <= joint.lower {
        (-std::f32::consts::PI, std::f32::consts::PI)
    } else {
        (joint.lower, joint.upper)
    }
}

fn root_link(model: &ParsedUrdf) -> String {
    for candidate in ["base_footprint", "base_link"] {
        if model.links.iter().any(|link| link.name == candidate) {
            return candidate.to_string();
        }
    }
    model
        .links
        .iter()
        .find(|link| !model.joints.iter().any(|joint| joint.child == link.name))
        .map(|link| link.name.clone())
        .unwrap_or_else(|| model.links[0].name.clone())
}

fn walk(
    model: &ParsedUrdf,
    link_name: &str,
    parent: &Xform,
    angles: &[(String, f32)],
    out: &mut Vec<LinkPose>,
    guard: &mut usize,
) {
    if *guard > 512 || out.iter().any(|pose| pose.name == link_name) {
        return;
    }
    *guard += 1;
    if let Some(link) = model.links.iter().find(|link| link.name == link_name) {
        let center = parent.apply(link.visual_origin);
        out.push(LinkPose {
            name: link.name.clone(),
            x: center[0],
            y: center[1],
            z: center[2],
            sx: link.size[0],
            sy: link.size[1],
            sz: link.size[2],
        });
    }
    for joint in model
        .joints
        .iter()
        .filter(|joint| joint.parent == link_name)
    {
        let angle = angles
            .iter()
            .find(|(name, _)| name == &joint.name)
            .map(|(_, value)| *value)
            .unwrap_or(0.0);
        let origin = Xform::from_xyz_rpy(joint.origin_xyz, joint.origin_rpy);
        let motion = match joint.joint_type.as_str() {
            "prismatic" => Xform::translate(scale(joint.axis, angle)),
            "revolute" | "continuous" => Xform::from_axis_angle(joint.axis, angle),
            _ => Xform::identity(),
        };
        let child = parent.mul(&origin).mul(&motion);
        walk(model, &joint.child, &child, angles, out, guard);
    }
}

fn parse_link(name: &str, node: roxmltree::Node<'_, '_>) -> UrdfLink {
    if let Some((size, origin)) = first_box(node, "collision").or_else(|| first_box(node, "visual"))
    {
        return UrdfLink {
            name: name.to_string(),
            size,
            visual_origin: origin,
        };
    }
    if let Some((size, origin)) = first_cylinder(node) {
        return UrdfLink {
            name: name.to_string(),
            size,
            visual_origin: origin,
        };
    }
    UrdfLink {
        name: name.to_string(),
        size: [0.04, 0.04, 0.04],
        visual_origin: [0.0, 0.0, 0.0],
    }
}

fn first_box(node: roxmltree::Node<'_, '_>, tag: &str) -> Option<([f32; 3], [f32; 3])> {
    for block in node
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == tag)
    {
        let origin = origin_xyz(block);
        if let Some(geometry) = child_named(block, "geometry") {
            if let Some(box_node) = child_named(geometry, "box") {
                let size = nums3(box_node.attribute("size").unwrap_or("0.04 0.04 0.04"));
                return Some((size, origin));
            }
        }
    }
    None
}

fn first_cylinder(node: roxmltree::Node<'_, '_>) -> Option<([f32; 3], [f32; 3])> {
    let block = node
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == "visual")?;
    let origin = origin_xyz(block);
    let geometry = child_named(block, "geometry")?;
    let cylinder = child_named(geometry, "cylinder")?;
    let radius = cylinder
        .attribute("radius")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.02);
    let length = cylinder
        .attribute("length")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.04);
    let diameter = radius * 2.0;
    Some(([diameter, diameter, length], origin))
}

fn parse_joint(node: roxmltree::Node<'_, '_>) -> Option<UrdfJoint> {
    let name = node.attribute("name")?.to_string();
    let joint_type = node.attribute("type").unwrap_or("fixed").to_string();
    let parent = child_named(node, "parent")?.attribute("link")?.to_string();
    let child = child_named(node, "child")?.attribute("link")?.to_string();
    let (origin_xyz, origin_rpy) =
        child_named(node, "origin").map_or(([0.0; 3], [0.0; 3]), |origin| {
            (
                nums3(origin.attribute("xyz").unwrap_or("0 0 0")),
                nums3(origin.attribute("rpy").unwrap_or("0 0 0")),
            )
        });
    let axis = child_named(node, "axis").map_or([0.0, 0.0, 1.0], |axis| {
        nums3(axis.attribute("xyz").unwrap_or("0 0 1"))
    });
    let (lower, upper) = child_named(node, "limit").map_or((0.0, 0.0), |limit| {
        (
            limit
                .attribute("lower")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            limit
                .attribute("upper")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
        )
    });
    Some(UrdfJoint {
        name,
        joint_type,
        parent,
        child,
        origin_xyz,
        origin_rpy,
        axis,
        lower,
        upper,
    })
}

fn child_named<'a>(node: roxmltree::Node<'a, 'a>, name: &str) -> Option<roxmltree::Node<'a, 'a>> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == name)
}

fn origin_xyz(node: roxmltree::Node<'_, '_>) -> [f32; 3] {
    child_named(node, "origin").map_or([0.0; 3], |origin| {
        nums3(origin.attribute("xyz").unwrap_or("0 0 0"))
    })
}

fn nums3(text: &str) -> [f32; 3] {
    let mut values = [0.0; 3];
    for (index, part) in text.split_whitespace().take(3).enumerate() {
        values[index] = part.parse().unwrap_or(0.0);
    }
    values
}

fn scale(axis: [f32; 3], amount: f32) -> [f32; 3] {
    let norm = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if norm < 1.0e-8 {
        return [0.0, 0.0, amount];
    }
    [
        axis[0] / norm * amount,
        axis[1] / norm * amount,
        axis[2] / norm * amount,
    ]
}

impl Xform {
    fn identity() -> Self {
        Self {
            r: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            t: [0.0, 0.0, 0.0],
        }
    }

    fn translate(t: [f32; 3]) -> Self {
        Self {
            r: Self::identity().r,
            t,
        }
    }

    fn from_xyz_rpy(xyz: [f32; 3], rpy: [f32; 3]) -> Self {
        Self {
            r: rpy_matrix(rpy[0], rpy[1], rpy[2]),
            t: xyz,
        }
    }

    fn from_axis_angle(axis: [f32; 3], angle: f32) -> Self {
        let norm = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if norm < 1.0e-8 || angle.abs() < 1.0e-8 {
            return Self::identity();
        }
        let x = axis[0] / norm;
        let y = axis[1] / norm;
        let z = axis[2] / norm;
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        Self {
            r: [
                [t * x * x + c, t * x * y - s * z, t * x * z + s * y],
                [t * x * y + s * z, t * y * y + c, t * y * z - s * x],
                [t * x * z - s * y, t * y * z + s * x, t * z * z + c],
            ],
            t: [0.0, 0.0, 0.0],
        }
    }

    fn mul(&self, other: &Self) -> Self {
        Self {
            r: mat_mul(self.r, other.r),
            t: add(mat_vec(self.r, other.t), self.t),
        }
    }

    fn apply(&self, point: [f32; 3]) -> [f32; 3] {
        add(mat_vec(self.r, point), self.t)
    }
}

fn rpy_matrix(roll: f32, pitch: f32, yaw: f32) -> [[f32; 3]; 3] {
    let rx = {
        let c = roll.cos();
        let s = roll.sin();
        [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
    };
    let ry = {
        let c = pitch.cos();
        let s = pitch.sin();
        [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
    };
    let rz = {
        let c = yaw.cos();
        let s = yaw.sin();
        [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
    };
    mat_mul(rz, mat_mul(ry, rx))
}

fn mat_mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

fn mat_vec(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    trait TestUnwrap<T> {
        fn expect_err_free(self) -> T;
        fn expect_pose(self) -> T;
    }

    impl<T> TestUnwrap<T> for Result<T, String> {
        fn expect_err_free(self) -> T {
            match self {
                Ok(value) => value,
                Err(err) => panic!("urdf parse failed: {err}"),
            }
        }
        fn expect_pose(self) -> T {
            match self {
                Ok(value) => value,
                Err(err) => panic!("missing pose: {err}"),
            }
        }
    }

    impl<T> TestUnwrap<T> for Option<T> {
        fn expect_err_free(self) -> T {
            match self {
                Some(value) => value,
                None => panic!("missing value"),
            }
        }
        fn expect_pose(self) -> T {
            match self {
                Some(value) => value,
                None => panic!("missing link pose"),
            }
        }
    }

    use super::*;

    const PLANAR: &str = r#"
        <robot name="fixture_arm">
          <link name="base"/>
          <link name="link1"/>
          <joint name="shoulder" type="revolute">
            <parent link="base"/>
            <child link="link1"/>
            <origin xyz="0 0 0.2" rpy="0 0 0"/>
            <axis xyz="0 1 0"/>
            <limit lower="-1.57" upper="1.57" effort="1" velocity="1"/>
          </joint>
          <link name="link2"/>
          <joint name="elbow" type="revolute">
            <parent link="link1"/>
            <child link="link2"/>
            <origin xyz="0.25 0 0" rpy="0 0 0"/>
            <axis xyz="0 1 0"/>
            <limit lower="-2" upper="2" effort="1" velocity="1"/>
          </joint>
        </robot>
    "#;

    #[test]
    fn zero_pose_places_elbow_along_x() {
        let model = parse_urdf(PLANAR).expect_err_free();
        let poses = pose_links(&model, &[]);
        let link2 = poses.iter().find(|pose| pose.name == "link2").expect_pose();
        assert!((link2.x - 0.25).abs() < 1.0e-4, "x {}", link2.x);
        assert!(link2.y.abs() < 1.0e-4);
        assert!((link2.z - 0.2).abs() < 1.0e-4, "z {}", link2.z);
    }

    #[test]
    fn shoulder_quarter_turn_drops_the_elbow() {
        let model = parse_urdf(PLANAR).expect_err_free();
        let poses = pose_links(
            &model,
            &[("shoulder".to_string(), std::f32::consts::FRAC_PI_2)],
        );
        let link2 = poses.iter().find(|pose| pose.name == "link2").expect_pose();
        assert!(link2.x.abs() < 1.0e-3, "x {}", link2.x);
        assert!((link2.z - (-0.05)).abs() < 1.0e-3, "z {}", link2.z);
    }

    #[test]
    fn nori_expanded_urdf_has_the_shoulder() {
        let path = default_nori_urdf();
        let read = std::fs::read_to_string(&path);
        assert!(read.is_ok(), "missing {}", path.display());
        let text = match read {
            Ok(text) => text,
            Err(_) => return,
        };
        let parsed = parse_urdf(&text);
        assert!(parsed.is_ok());
        let model = match parsed {
            Ok(model) => model,
            Err(_) => return,
        };
        assert_eq!(model.name, "nori");
        assert!(model
            .joints
            .iter()
            .any(|joint| joint.name == "left_shoulder_pitch_joint"));
        assert!(pose_links(&model, &[]).len() > 10);
    }
}
