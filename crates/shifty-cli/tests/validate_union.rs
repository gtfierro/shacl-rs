use std::process::Command;

#[test]
fn version_subcommand_prints_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .arg("version")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn validation_rejects_empty_shapes() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-empty-shapes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    let data = dir.join("data.ttl");
    std::fs::write(&shapes, "").unwrap();
    std::fs::write(&data, "@prefix ex: <http://ex/> . ex:item ex:p ex:v .").unwrap();

    let rejected = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("explicit shapes graph is empty"));

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn validation_rejects_invalid_shapes_diagnostics() {
    let dir =
        std::env::temp_dir().join(format!("shifty-cli-invalid-shapes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    let data = dir.join("data.ttl");
    std::fs::write(
        &shapes,
        r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://ex/> .
            ex:S a sh:NodeShape ;
                sh:targetNode ex:item ;
                sh:sparql [
                    sh:select "SELECT $this WHERE { $this missing:p ?value }"
                ] .
        "#,
    )
    .unwrap();
    std::fs::write(&data, "@prefix ex: <http://ex/> . ex:item ex:p ex:value .").unwrap();

    let rejected = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("invalid SPARQL query"));

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn validation_executes_over_data_and_shapes_union() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-union-class-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    let data = dir.join("data.ttl");

    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:Parent a rdfs:Class, sh:NodeShape ;
                sh:property [ sh:path ex:forbidden ; sh:maxCount 0 ] .
            ex:Child rdfs:subClassOf ex:Parent .
            ex:shapeItem a ex:Child ; ex:forbidden ex:value .
        "#,
    )
    .unwrap();
    std::fs::write(
        &data,
        r#"
            @prefix ex: <http://ex/> .
            ex:item a ex:Child ; ex:forbidden ex:value .
        "#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("conforms: false"), "stdout: {stdout}");
    // The report spells data-graph nodes in the document's own vocabulary, so
    // the negative assertion has to use the compacted form too or it passes
    // whatever the output says.
    assert!(stdout.contains("ex:item"), "stdout: {stdout}");
    assert!(!stdout.contains("shapeItem"), "stdout: {stdout}");

    let data_only = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
            "--graph-mode",
            "data",
        ])
        .output()
        .unwrap();
    let data_stdout = String::from_utf8(data_only.stdout).unwrap();
    assert!(
        data_stdout.contains("conforms: true"),
        "stdout: {data_stdout}"
    );

    let union_all = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
            "--graph-mode",
            "union-all",
        ])
        .output()
        .unwrap();
    let union_all_stdout = String::from_utf8(union_all.stdout).unwrap();
    assert!(
        union_all_stdout.contains("ex:item"),
        "stdout: {union_all_stdout}"
    );
    assert!(
        union_all_stdout.contains("ex:shapeItem"),
        "stdout: {union_all_stdout}"
    );

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn sparql_constraints_see_the_shapes_graph() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-union-sparql-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    let data = dir.join("data.ttl");

    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:enabled ex:value true .
            ex:S a sh:NodeShape ;
                sh:targetNode ex:item ;
                sh:sparql [
                    sh:select "SELECT $this WHERE { ex:enabled ex:value true }"
                ] .
        "#,
    )
    .unwrap();
    std::fs::write(&data, "@prefix ex: <http://ex/> . ex:item ex:value 1 .").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("conforms: false"), "stdout: {stdout}");
    assert!(
        stdout.contains("SPARQL constraint at") && stdout.contains("not satisfied"),
        "stdout: {stdout}"
    );

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn inference_executes_over_data_and_shapes_union() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-infer-union-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    let data = dir.join("data.ttl");

    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:S a sh:NodeShape ;
                sh:targetClass ex:Thing ;
                sh:rule [
                    a sh:SPARQLRule ;
                    sh:construct """
                        CONSTRUCT { ?object ?inverse $this }
                        WHERE {
                            $this ?predicate ?object .
                            ?predicate ex:inverseOf ?inverse .
                        }
                    """
                ] .
            ex:p ex:inverseOf ex:q .
        "#,
    )
    .unwrap();
    std::fs::write(
        &data,
        r#"
            @prefix ex: <http://ex/> .
            ex:a a ex:Thing ; ex:p ex:b .
        "#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "infer",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("inferred 1 triple(s)"), "stdout: {stdout}");
    assert!(
        stdout.contains("<http://ex/b> <http://ex/q> <http://ex/a>"),
        "stdout: {stdout}"
    );

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn validation_runs_inference_first() {
    let dir =
        std::env::temp_dir().join(format!("shifty-cli-validate-infer-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    let data = dir.join("data.ttl");

    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:S a sh:NodeShape ;
                sh:targetNode ex:item ;
                sh:property [ sh:path ex:derived ; sh:maxCount 0 ] ;
                sh:rule [
                    a sh:TripleRule ;
                    sh:subject sh:this ;
                    sh:predicate ex:derived ;
                    sh:object ex:value
                ] .
        "#,
    )
    .unwrap();
    std::fs::write(
        &data,
        "@prefix ex: <http://ex/> . ex:item ex:input ex:value .",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("conforms: false"), "stdout: {stdout}");
    // The inferred triple is what busts the bound, so the count has to see it.
    assert!(
        stdout.contains("found        1 value(s) along the path; at most 0 allowed"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("requirement  ∄ ex:derived"),
        "stdout: {stdout}"
    );

    let no_infer = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
            "--no-infer",
        ])
        .output()
        .unwrap();
    assert!(
        no_infer.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&no_infer.stderr)
    );
    let no_infer_stdout = String::from_utf8(no_infer.stdout).unwrap();
    assert!(
        no_infer_stdout.contains("conforms: true"),
        "stdout: {no_infer_stdout}"
    );

    let embedded_no_infer = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--no-infer",
        ])
        .output()
        .unwrap();
    assert!(
        embedded_no_infer.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&embedded_no_infer.stderr)
    );
    let embedded_no_infer_stdout = String::from_utf8(embedded_no_infer.stdout).unwrap();
    assert!(
        embedded_no_infer_stdout.contains("conforms: true"),
        "stdout: {embedded_no_infer_stdout}"
    );

    let embedded_report_no_infer = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--no-infer",
            "--report",
        ])
        .output()
        .unwrap();
    assert!(
        embedded_report_no_infer.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&embedded_report_no_infer.stderr)
    );
    let embedded_report_no_infer_stdout =
        String::from_utf8(embedded_report_no_infer.stdout).unwrap();
    assert!(
        embedded_report_no_infer_stdout.contains("sh:conforms true"),
        "stdout: {embedded_report_no_infer_stdout}"
    );

    let report = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--data",
            data.to_str().unwrap(),
            "--report",
        ])
        .output()
        .unwrap();
    assert!(
        report.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&report.stderr)
    );
    let report_stdout = String::from_utf8(report.stdout).unwrap();
    assert!(
        report_stdout.contains("sh:conforms false"),
        "stdout: {report_stdout}"
    );
    assert!(
        report_stdout.contains("sh:sourceConstraintComponent sh:MaxCountConstraintComponent"),
        "stdout: {report_stdout}"
    );

    std::fs::remove_dir_all(dir).unwrap();
}

/// The two nodes in a reason are the thing a first-time reader confuses: the
/// focus node was selected for checking, the value node was reached from it
/// along the path and is what actually failed. Both must be named.
#[test]
fn text_report_labels_the_focus_and_value_nodes() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-labels-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:S a sh:NodeShape ;
                sh:targetClass ex:T ;
                sh:property [ sh:path ex:p ; sh:class ex:Wanted ] .

            ex:a a ex:T ; ex:p ex:wrong .
        "#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args(["validate", "--shapes", shapes.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("Finding 1 of 1"), "stdout: {stdout}");
    assert!(stdout.contains("affects      ex:a"), "stdout: {stdout}");
    assert!(stdout.contains("value node   ex:wrong"), "stdout: {stdout}");
    assert!(stdout.contains("path         ex:p"), "stdout: {stdout}");
    assert!(
        stdout.contains("target       class(ex:T)"),
        "stdout: {stdout}"
    );
    // The notation key explains only the symbols this report actually used.
    assert!(stdout.contains("∀ p . X"), "stdout: {stdout}");
    assert!(!stdout.contains("∄ p"), "stdout: {stdout}");

    std::fs::remove_dir_all(dir).unwrap();
}

/// JSON keeps the raw algebra, whose child links are arena ids. Those ids have
/// to resolve inside the document, or a consumer is stuck exactly where `@257`
/// left a reader of the text report.
#[test]
fn json_report_resolves_every_constraint_pointer() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-json-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:S a sh:NodeShape ;
                sh:targetClass ex:T ;
                sh:property [
                    sh:path ex:p ;
                    sh:qualifiedValueShape [ sh:class ex:Wanted ] ;
                    sh:qualifiedMinCount 1 ;
                ] .

            ex:a a ex:T .
        "#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args([
            "validate",
            "--shapes",
            shapes.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let violation = &doc["violations"][0];
    assert_eq!(violation["target"], "class(ex:T)");
    assert_eq!(violation["shape_name"], "http://ex/S");

    let reason = &violation["reasons"][0];
    assert_eq!(reason["observed_count"], 0);
    // The constraint in words, so a consumer needs no arena at all …
    assert_eq!(reason["definition"], "∃[1..] ex:p . instance of ex:Wanted");

    // … and for one that walks the algebra, every id it can reach resolves.
    let shapes_map = doc["shapes"].as_object().expect("shapes map");
    let mut stack = vec![reason["constraint_id"].as_u64().unwrap()];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let shape = shapes_map
            .get(&id.to_string())
            .unwrap_or_else(|| panic!("slot {id} referenced but not shipped: {shapes_map:?}"));
        let text = shape.to_string();
        // Child links appear as bare integers in the serialized algebra.
        for id in shapes_map.keys() {
            if text.contains(&format!(":{id}")) || text.contains(&format!("[{id}")) {
                stack.push(id.parse().unwrap());
            }
        }
    }
    assert!(seen.len() > 1, "expected a nested constraint: {seen:?}");
    // Only what the report reaches, not the whole arena.
    assert!(shapes_map.len() < 40, "shipped {} slots", shapes_map.len());

    std::fs::remove_dir_all(dir).unwrap();
}

/// The same constraint failing on many nodes is one thing wrong with the graph.
/// Printing its explanation once per node buries whatever else is wrong, so
/// violations that render identically are grouped and the nodes listed together.
#[test]
fn text_report_groups_violations_that_share_a_finding() {
    let dir = std::env::temp_dir().join(format!("shifty-cli-group-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shapes = dir.join("shapes.ttl");
    std::fs::write(
        &shapes,
        r#"
            @prefix ex: <http://ex/> .
            @prefix sh: <http://www.w3.org/ns/shacl#> .

            ex:S a sh:NodeShape ;
                sh:targetClass ex:T ;
                sh:property [ sh:path ex:p ; sh:minCount 1 ] .

            ex:Other a sh:NodeShape ;
                sh:targetClass ex:T ;
                sh:property [ sh:path ex:q ; sh:nodeKind sh:IRI ] .

            ex:a a ex:T ; ex:q "literal" .
            ex:b a ex:T ; ex:q "literal" .
            ex:c a ex:T ; ex:q "literal" .
        "#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shifty"))
        .args(["validate", "--shapes", shapes.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Three nodes, each failing the same two shapes: six violations, two findings.
    assert!(
        stdout.contains("conforms: false — 6 violations in 2 findings"),
        "stdout: {stdout}"
    );
    // Each explanation appears once, not once per node.
    assert_eq!(stdout.matches("requirement").count(), 2, "stdout: {stdout}");
    assert!(
        stdout.contains("affects      3 focus nodes"),
        "stdout: {stdout}"
    );
    for node in ["ex:a", "ex:b", "ex:c"] {
        assert!(stdout.contains(node), "missing {node}: {stdout}");
    }
    // A grouped value node has to say what it is. Grouping by reason rather than
    // by violation is what makes that possible: one value per node per finding,
    // so the heading can name the column instead of leaving bare parentheses for
    // the reader to decode.
    assert!(
        stdout.contains("focus nodes, each with the value node that failed"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("(\"literal\","), "bare values: {stdout}");

    std::fs::remove_dir_all(dir).unwrap();
}
