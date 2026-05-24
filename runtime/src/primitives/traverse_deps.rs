//! `traverse-deps` — verify spec dependencies exist as directories and that
//! their pipeline status is compatible, and surface any cycles in the
//! reachable dep subgraph.
//!
//! Compatibility rule: a dependency is compatible when its `status` is
//! `planned`, `in-progress`, or `done`. Pipeline status `draft` or
//! `clarified` blocks consumers because there is no committed plan to build
//! against; absent dependency directories block always.
//!
//! Cycle detection (added by spec 022's `traverse-deps-cycle-check`
//! scenario): the primitive walks every reachable spec from the targeted
//! feature, reads each visited spec's frontmatter `dependencies`, and runs
//! Tarjan's strongly-connected-components algorithm over the resulting
//! subgraph. Any non-trivial SCC — size ≥ 2, or a self-loop — surfaces in
//! the result's `cycles` field. Cycle detection is defense-in-depth that
//! complements spec 017's `gen-spec-deps.sh` generator-side cycle check;
//! the primitive fires when the upstream generator was bypassed, the
//! adopter is on an older shipped script, or stale frontmatter edits
//! re-introduce a cycle outside the generator's purview.

use std::collections::HashMap;
use std::path::Path;

use crate::primitives::{PrimitiveError, Result, read_text, split_frontmatter};
use crate::schema::primitives::{
    DependencyEdge, Frontmatter, TraverseDepsArgs, TraverseDepsResult,
};

const COMPATIBLE_STATUSES: &[&str] = &["planned", "in-progress", "done"];

/// Execute the `traverse-deps` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent, [`PrimitiveError::Io`] / [`PrimitiveError::MissingFrontmatter`]
/// on read or parse failures of the feature's own `spec.md`, and
/// [`PrimitiveError::Yaml`] when its frontmatter is malformed. Missing or
/// malformed dependency specs are reported as findings, not errors;
/// likewise the cycle walker tolerates missing or malformed downstream
/// nodes by treating them as sinks (no outgoing edges).
pub fn run(args: &TraverseDepsArgs, repo: &Path) -> Result<TraverseDepsResult> {
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_yaml::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;

    let mut edges: Vec<DependencyEdge> = Vec::with_capacity(frontmatter.dependencies.len());
    let mut overall = true;
    for dep_name in &frontmatter.dependencies {
        let dep_dir = repo.join("specs").join(dep_name);
        let dep_spec = dep_dir.join("spec.md");
        let exists = dep_dir.is_dir() && dep_spec.is_file();
        let status = if exists {
            read_status(&dep_spec).unwrap_or_default()
        } else {
            String::new()
        };
        let compatible = exists && COMPATIBLE_STATUSES.contains(&status.as_str());
        if !compatible {
            overall = false;
        }
        edges.push(DependencyEdge {
            feature: dep_name.clone(),
            exists,
            status,
            compatible,
        });
    }

    let cycles = detect_cycles(repo, &args.feature);
    if !cycles.is_empty() {
        overall = false;
    }

    Ok(TraverseDepsResult {
        dependencies: edges,
        compatible: overall,
        cycles,
    })
}

fn read_status(spec_path: &Path) -> Result<String> {
    let content = read_text(spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, spec_path)?;
    let parsed: Frontmatter =
        serde_yaml::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.into(),
            source,
        })?;
    Ok(parsed.status)
}

/// Read a spec's frontmatter `dependencies` list, returning an empty Vec
/// when the file is absent, unreadable, or has malformed frontmatter. The
/// cycle walker tolerates these failure modes — they degrade the node to
/// a sink in the reachable subgraph without halting the primitive.
fn read_dependencies(spec_path: &Path) -> Vec<String> {
    if !spec_path.is_file() {
        return Vec::new();
    }
    let Ok(content) = read_text(spec_path) else {
        return Vec::new();
    };
    let Ok((fm_text, _)) = split_frontmatter(&content, spec_path) else {
        return Vec::new();
    };
    serde_yaml::from_str::<Frontmatter>(fm_text)
        .map(|fm| fm.dependencies)
        .unwrap_or_default()
}

/// Walk the reachable dep subgraph from `start`, then run Tarjan's SCC
/// over the resulting directed graph. Returns one entry per non-trivial
/// SCC: size ≥ 2 (multi-node cycle) or size 1 with a self-edge.
fn detect_cycles(repo: &Path, start: &str) -> Vec<Vec<String>> {
    let mut order: Vec<String> = Vec::new();
    let mut index_of: HashMap<String, usize> = HashMap::new();
    let mut adj: Vec<Vec<usize>> = Vec::new();

    visit(start, repo, &mut order, &mut index_of, &mut adj);

    let mut tarjan = Tarjan::new(&adj);
    tarjan.run();

    let mut cycles: Vec<Vec<String>> = Vec::new();
    for scc in tarjan.sccs {
        let is_cycle = scc.len() >= 2
            || (scc.len() == 1 && {
                let v = scc[0];
                adj[v].contains(&v)
            });
        if is_cycle {
            cycles.push(scc.into_iter().map(|i| order[i].clone()).collect());
        }
    }
    cycles
}

fn visit(
    node: &str,
    repo: &Path,
    order: &mut Vec<String>,
    index_of: &mut HashMap<String, usize>,
    adj: &mut Vec<Vec<usize>>,
) -> usize {
    if let Some(&idx) = index_of.get(node) {
        return idx;
    }
    let idx = order.len();
    order.push(node.to_string());
    index_of.insert(node.to_string(), idx);
    adj.push(Vec::new());

    let spec_path = repo.join("specs").join(node).join("spec.md");
    let deps = read_dependencies(&spec_path);
    for dep in deps {
        let dep_idx = visit(&dep, repo, order, index_of, adj);
        adj[idx].push(dep_idx);
    }
    idx
}

/// Tarjan's strongly-connected-components algorithm. Recursive form; the
/// dep graphs the primitive walks are bounded by the spec corpus (tens of
/// nodes in practice), so stack depth is not a concern.
struct Tarjan<'a> {
    adj: &'a [Vec<usize>],
    indices: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    counter: usize,
    sccs: Vec<Vec<usize>>,
}

impl<'a> Tarjan<'a> {
    fn new(adj: &'a [Vec<usize>]) -> Self {
        let n = adj.len();
        Self {
            adj,
            indices: vec![None; n],
            lowlinks: vec![0; n],
            on_stack: vec![false; n],
            stack: Vec::new(),
            counter: 0,
            sccs: Vec::new(),
        }
    }

    fn run(&mut self) {
        for v in 0..self.adj.len() {
            if self.indices[v].is_none() {
                self.strongconnect(v);
            }
        }
    }

    fn strongconnect(&mut self, v: usize) {
        let v_index = self.counter;
        self.indices[v] = Some(v_index);
        self.lowlinks[v] = v_index;
        self.counter += 1;
        self.stack.push(v);
        self.on_stack[v] = true;

        for i in 0..self.adj[v].len() {
            let w = self.adj[v][i];
            if self.indices[w].is_none() {
                self.strongconnect(w);
                self.lowlinks[v] = self.lowlinks[v].min(self.lowlinks[w]);
            } else if self.on_stack[w]
                && let Some(w_index) = self.indices[w]
            {
                self.lowlinks[v] = self.lowlinks[v].min(w_index);
            }
        }

        if self.lowlinks[v] == v_index {
            let mut component: Vec<usize> = Vec::new();
            while let Some(w) = self.stack.pop() {
                self.on_stack[w] = false;
                component.push(w);
                if w == v {
                    break;
                }
            }
            // Tarjan emits SCC members in pop order (root last); reverse so
            // the slugs appear in traversal order — first node entered
            // first, matching the scenario's "slugs in traversal order"
            // expectation.
            component.reverse();
            self.sccs.push(component);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::path::PathBuf;

    fn fixture_repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
    }

    fn write_spec(dir: &Path, slug: &str, status: &str, deps: &[&str]) {
        let spec_dir = dir.join("specs").join(slug);
        std::fs::create_dir_all(&spec_dir).unwrap();
        let dep_list = if deps.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", deps.join(", "))
        };
        std::fs::write(
            spec_dir.join("spec.md"),
            format!("---\nstatus: {status}\ndependencies: {dep_list}\n---\n\n# {slug}\n"),
        )
        .unwrap();
    }

    #[test]
    fn dependent_resolves_basic_edge() {
        let repo = fixture_repo();
        let result = run(
            &TraverseDepsArgs {
                feature: "002-dependent".into(),
            },
            &repo,
        )
        .unwrap();
        assert_eq!(result.dependencies.len(), 1);
        let edge = &result.dependencies[0];
        assert_eq!(edge.feature, "001-basic");
        assert!(edge.exists);
        assert_eq!(edge.status, "clarified");
        // 001-basic is `clarified`, which is not in the compatible set —
        // dependents need at least `planned` upstream, so the edge (and
        // overall result) reports incompatible.
        assert!(!edge.compatible);
        assert!(!result.compatible);
        assert!(result.cycles.is_empty());
    }

    #[test]
    fn planned_dependency_is_compatible() {
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "100-upstream", "planned", &[]);
        write_spec(tmp.path(), "101-downstream", "planned", &["100-upstream"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "101-downstream".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.compatible);
        assert_eq!(result.dependencies.len(), 1);
        assert!(result.dependencies[0].compatible);
        assert!(result.cycles.is_empty());
    }

    #[test]
    fn missing_dependency_is_incompatible() {
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "003-missing-dep", "planned", &["999-nope"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "003-missing-dep".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible);
        assert_eq!(result.dependencies.len(), 1);
        let edge = &result.dependencies[0];
        assert_eq!(edge.feature, "999-nope");
        assert!(!edge.exists);
        assert!(edge.status.is_empty());
        assert!(!edge.compatible);
        assert!(result.cycles.is_empty());
    }

    #[test]
    fn empty_dependencies_is_compatible() {
        let repo = fixture_repo();
        let result = run(
            &TraverseDepsArgs {
                feature: "001-basic".into(),
            },
            &repo,
        )
        .unwrap();
        assert!(result.compatible);
        assert!(result.dependencies.is_empty());
        assert!(result.cycles.is_empty());
    }

    // -- Cycle-detection coverage (scenario: traverse-deps-cycle-check) ----

    #[test]
    fn two_cycle_among_planned_specs_reports_scc() {
        // A ↔ B (two-node cycle); both planned so the edge-existence and
        // status-compatibility checks pass cleanly and the cycle is the
        // sole reason `compatible` flips to false.
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "200-a", "planned", &["201-b"]);
        write_spec(tmp.path(), "201-b", "planned", &["200-a"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "200-a".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible, "cycle must flip overall to false");
        assert_eq!(result.cycles.len(), 1);
        let cycle = &result.cycles[0];
        assert_eq!(cycle.len(), 2);
        assert!(cycle.contains(&"200-a".to_string()));
        assert!(cycle.contains(&"201-b".to_string()));
    }

    #[test]
    fn self_cycle_is_reported_as_one_cycle() {
        // Per the scenario's edge case: a spec listing itself surfaces as
        // a 1-cycle. Without the self-edge check Tarjan's would silently
        // drop the singleton SCC.
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "300-self", "planned", &["300-self"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "300-self".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible);
        assert_eq!(result.cycles.len(), 1);
        assert_eq!(result.cycles[0], vec!["300-self".to_string()]);
    }

    #[test]
    fn multiple_disjoint_cycles_each_surface_as_their_own_scc() {
        // Targeted feature M depends on A and X; A↔B and X↔Y form two
        // independent cycles. The primitive reports both SCCs in one pass.
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "400-m", "planned", &["401-a", "403-x"]);
        write_spec(tmp.path(), "401-a", "planned", &["402-b"]);
        write_spec(tmp.path(), "402-b", "planned", &["401-a"]);
        write_spec(tmp.path(), "403-x", "planned", &["404-y"]);
        write_spec(tmp.path(), "404-y", "planned", &["403-x"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "400-m".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible);
        assert_eq!(result.cycles.len(), 2);
        let mut slugs: Vec<String> = result.cycles.iter().flatten().cloned().collect();
        slugs.sort();
        assert_eq!(
            slugs,
            vec![
                "401-a".to_string(),
                "402-b".to_string(),
                "403-x".to_string(),
                "404-y".to_string(),
            ]
        );
    }

    #[test]
    fn missing_node_does_not_close_a_cycle() {
        // Per the scenario's edge case: a missing dependency that would
        // close a cycle if present produces a missing-dep finding but no
        // cycle finding — the closing edge is absent in the walked graph.
        // A → B → A (would-be cycle through B), but B is missing.
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "500-a", "planned", &["501-missing"]);
        // 501-missing has no spec on disk.
        let result = run(
            &TraverseDepsArgs {
                feature: "500-a".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible, "missing-dep finding still fires");
        assert_eq!(result.dependencies.len(), 1);
        assert!(!result.dependencies[0].exists);
        assert!(
            result.cycles.is_empty(),
            "no cycle when closing edge is absent"
        );
    }

    #[test]
    fn cycle_entirely_among_done_specs_is_still_reported() {
        // Per the scenario's edge case: cycles among `done` specs are
        // structural defects in the artifact regardless of operational
        // state. Aligns with spec 017's same-status posture on the
        // generator-side check.
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "600-a", "done", &["601-b"]);
        write_spec(tmp.path(), "601-b", "done", &["600-a"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "600-a".into(),
            },
            tmp.path(),
        )
        .unwrap();
        // Each direct edge is status-compatible (done), so the edge-level
        // check would have passed — only the cycle flips `compatible`.
        assert!(result.dependencies[0].compatible);
        assert!(!result.compatible);
        assert_eq!(result.cycles.len(), 1);
    }

    #[test]
    fn stale_frontmatter_cycle_is_visible_to_the_primitive() {
        // Per the scenario's edge case: when frontmatter `dependencies`
        // lists an edge no longer present in the body, `traverse-deps`
        // walks the frontmatter and sees the cycle. Resolution path:
        // re-run `gen-spec-deps.sh` to remove the stale edge. Here we
        // simulate the stale state by writing a frontmatter cycle
        // directly (no body links involved at the primitive layer).
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "700-a", "planned", &["701-b"]);
        write_spec(tmp.path(), "701-b", "planned", &["700-a"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "700-a".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.cycles.len(), 1);
    }

    #[test]
    fn three_node_cycle_via_intermediate_node() {
        // A → B → C → A reports one 3-node SCC. Asserts the algorithm
        // handles cycles deeper than one hop from the targeted feature.
        let tmp = tempfile::tempdir().unwrap();
        write_spec(tmp.path(), "800-a", "planned", &["801-b"]);
        write_spec(tmp.path(), "801-b", "planned", &["802-c"]);
        write_spec(tmp.path(), "802-c", "planned", &["800-a"]);
        let result = run(
            &TraverseDepsArgs {
                feature: "800-a".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible);
        assert_eq!(result.cycles.len(), 1);
        assert_eq!(result.cycles[0].len(), 3);
    }
}
