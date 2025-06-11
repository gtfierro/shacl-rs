use shacl_engine::context::ValidationContext;
use oxigraph::graph::Graph;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::vocab::{rdf, sh};
use oxigraph::model::*;
use oxigraph::store::Store;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Vocabulary for SHACL Test Suite
struct SHT {
    validate: NamedNode,
    data_graph: NamedNode,
    shapes_graph: NamedNode,
    approved: NamedNode,
    proposed: NamedNode,
    rejected: NamedNode,
    failure: NamedNode,
}

impl SHT {
    fn new() -> Self {
        Self {
            validate: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#Validate"),
            data_graph: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#dataGraph"),
            shapes_graph: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#shapesGraph"),
            approved: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#approved"),
            proposed: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#proposed"),
            rejected: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#rejected"),
            failure: NamedNode::new_unchecked("http://www.w3.org/ns/shacl-test#Failure"),
        }
    }
}

// Vocabulary for Test Manifest
struct MF {
    manifest: NamedNode,
    entries: NamedNode,
    name: NamedNode,
    action: NamedNode,
    result: NamedNode,
    status: NamedNode,
}

impl MF {
    fn new() -> Self {
        Self {
            manifest: NamedNode::new_unchecked(
                "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#Manifest",
            ),
            entries: NamedNode::new_unchecked(
                "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#entries",
            ),
            name: NamedNode::new_unchecked(
                "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#name",
            ),
            action: NamedNode::new_unchecked(
                "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#action",
            ),
            result: NamedNode::new_unchecked(
                "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#result",
            ),
            status: NamedNode::new_unchecked(
                "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#status",
            ),
        }
    }
}

fn find_manifest_files(base_dir: &str) -> Vec<PathBuf> {
    WalkDir::new(base_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "ttl")
        })
        .map(|e| e.into_path())
        .collect()
}

fn load_graph_from_path(
    file_path: &Path,
) -> Result<Store, Box<dyn Error>> {
    let store = Store::new()?;
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file '{}': {}", file_path.display(), e))?;
    let reader = BufReader::new(file);
    let parser = RdfParser::from_format(RdfFormat::Turtle);
    store.bulk_loader().load_from_reader(parser, reader)?;
    Ok(store)
}

fn parse_rdf_list(store: &Store, list_head: Term) -> Vec<Term> {
    let mut items = Vec::new();
    let mut current = list_head;

    while current != rdf::NIL.into() {
        if let Some(subject_ref) = current.as_subject_ref() {
            if let Ok(Some(item)) =
                store.object_for_subject_predicate(subject_ref, rdf::FIRST, GraphName::DefaultGraph)
            {
                items.push(item.into_term());
            }
            if let Ok(Some(next)) =
                store.object_for_subject_predicate(subject_ref, rdf::REST, GraphName::DefaultGraph)
            {
                current = next.into_term();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    items
}

fn extract_expected_report(manifest_store: &Store, result_node: SubjectRef) -> Graph {
    let mut report_graph = Graph::new();

    for quad in manifest_store
        .quads_for_pattern(Some(result_node), None, None, Some(GraphName::DefaultGraph.into()))
    {
        report_graph.insert(&quad.unwrap().into());
    }

    let sh_results = manifest_store
        .objects_for_subject_predicate(result_node, sh::RESULT, GraphName::DefaultGraph)
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    for res in sh_results {
        if let Some(res_subject) = res.as_subject_ref() {
            for quad in manifest_store.quads_for_pattern(
                Some(res_subject),
                None,
                None,
                Some(GraphName::DefaultGraph.into()),
            ) {
                let quad = quad.unwrap();
                report_graph.insert(&quad.into());

                if quad.predicate == sh::RESULT_PATH {
                    recursively_add_path(&mut report_graph, manifest_store, quad.object.clone());
                }
            }
        }
    }

    report_graph
}

fn recursively_add_path(report_graph: &mut Graph, manifest_store: &Store, path_node: Term) {
    if let Some(path_subject) = path_node.as_subject_ref() {
        for quad in manifest_store.quads_for_pattern(
            Some(path_subject),
            None,
            None,
            Some(GraphName::DefaultGraph.into()),
        ) {
            let quad = quad.unwrap();
            if report_graph.insert(&quad.into()) {
                recursively_add_path(report_graph, manifest_store, quad.object.clone());
            }
        }
    }
}

#[test]
fn run_w3c_shacl_test_suite() {
    let mf = MF::new();
    let sht = SHT::new();

    let manifest_paths = find_manifest_files("tests/test-suite");
    if manifest_paths.is_empty() {
        println!("Warning: No W3C SHACL test suite files found in 'tests/test-suite'. Skipping tests.");
        return;
    }

    for manifest_path in manifest_paths {
        println!("Running tests from manifest: {}", manifest_path.display());
        let manifest_store = match load_graph_from_path(&manifest_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to load manifest {}: {}", manifest_path.display(), e);
                continue;
            }
        };

        let manifest_subjects = manifest_store
            .subjects_for_predicate_object(rdf::TYPE, mf.manifest.as_ref(), GraphName::DefaultGraph)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        for manifest_subject in manifest_subjects {
            let entries_list_head = manifest_store
                .object_for_subject_predicate(
                    manifest_subject.as_ref(),
                    mf.entries.as_ref(),
                    GraphName::DefaultGraph,
                )
                .unwrap()
                .unwrap();
            let entries = parse_rdf_list(&manifest_store, entries_list_head.into_term());

            for entry in entries {
                let entry_subject = entry.as_subject_ref().unwrap();
                let entry_name = manifest_store
                    .object_for_subject_predicate(entry_subject, mf.name.as_ref(), GraphName::DefaultGraph)
                    .unwrap()
                    .unwrap();
                let entry_name_lit = entry_name.as_literal().unwrap();
                println!("  Running test: {}", entry_name_lit.value());

                let status = manifest_store
                    .object_for_subject_predicate(entry_subject, mf.status.as_ref(), GraphName::DefaultGraph)
                    .unwrap()
                    .unwrap();
                if status == sht.rejected.as_ref().into() {
                    println!("    SKIPPING rejected test.");
                    continue;
                }

                let test_type = manifest_store
                    .object_for_subject_predicate(entry_subject, rdf::TYPE, GraphName::DefaultGraph)
                    .unwrap()
                    .unwrap();
                if test_type != sht.validate.as_ref().into() {
                    println!("    SKIPPING test of type: {}", test_type);
                    continue;
                }

                let action_node = manifest_store
                    .object_for_subject_predicate(entry_subject, mf.action.as_ref(), GraphName::DefaultGraph)
                    .unwrap()
                    .unwrap();
                let shapes_graph_term = manifest_store
                    .object_for_subject_predicate(
                        action_node.as_subject_ref().unwrap(),
                        sht.shapes_graph.as_ref(),
                        GraphName::DefaultGraph,
                    )
                    .unwrap()
                    .unwrap();
                let data_graph_term = manifest_store
                    .object_for_subject_predicate(
                        action_node.as_subject_ref().unwrap(),
                        sht.data_graph.as_ref(),
                        GraphName::DefaultGraph,
                    )
                    .unwrap()
                    .unwrap();

                let manifest_dir = manifest_path.parent().unwrap();
                
                let shapes_graph_str = shapes_graph_term.as_named_node().unwrap().as_str();
                let shapes_graph_path = if shapes_graph_str.is_empty() { manifest_path.clone() } else { manifest_dir.join(shapes_graph_str) };

                let data_graph_str = data_graph_term.as_named_node().unwrap().as_str();
                let data_graph_path = if data_graph_str.is_empty() { manifest_path.clone() } else { manifest_dir.join(data_graph_str) };

                let context_result = ValidationContext::from_files(
                    shapes_graph_path.to_str().unwrap(),
                    data_graph_path.to_str().unwrap(),
                );

                let result_node = manifest_store
                    .object_for_subject_predicate(entry_subject, mf.result.as_ref(), GraphName::DefaultGraph)
                    .unwrap()
                    .unwrap();

                if result_node == sht.failure.as_ref().into() {
                    if context_result.is_err() {
                        println!("    PASS (expected failure, and got one)");
                    } else {
                        panic!(
                            "Test '{}' FAILED: Expected a failure, but validation succeeded.",
                            entry_name_lit.value()
                        );
                    }
                    continue;
                }

                let context = match context_result {
                    Ok(ctx) => ctx,
                    Err(e) => panic!(
                        "Test '{}' FAILED: validation returned an unexpected error: {}",
                        entry_name_lit.value(),
                        e
                    ),
                };

                let report_builder = context.validate();
                let actual_report_graph = report_builder.to_graph(&context);

                let expected_report_graph =
                    extract_expected_report(&manifest_store, result_node.as_subject_ref().unwrap());

                // NOTE: The SHACL test suite spec requires normalization of the report graphs before comparison.
                // This implementation performs a direct isomorphism check, which is a good baseline but may fail
                // for some complex cases that require the specified normalization steps.
                if actual_report_graph.is_isomorphic(&expected_report_graph) {
                    println!("    PASS");
                } else {
                    eprintln!("Test '{}' FAILED: Reports are not isomorphic.", entry_name_lit.value());
                    eprintln!("------- EXPECTED REPORT -------");
                    eprintln!("{}", expected_report_graph);
                    eprintln!("-------- ACTUAL REPORT --------");
                    eprintln!("{}", actual_report_graph);
                    eprintln!("-----------------------------");
                    panic!("Test failed: {}", entry_name_lit.value());
                }
            }
        }
    }
}
