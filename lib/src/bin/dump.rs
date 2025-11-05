use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{Graph, Triple};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn load_graph(path: &str) -> Result<Graph, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let parser = RdfParser::from_format(RdfFormat::Turtle).without_named_graphs();
    let mut graph = Graph::new();
    for quad in parser.for_reader(reader) {
        let triple: Triple = quad?.into();
        graph.insert(triple.as_ref());
    }
    Ok(graph)
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect("path required");
    let graph = load_graph(&path)?;
    println!("triples: {}", graph.len());
    for triple in graph.iter() {
        println!(
            "{} {} {} .",
            triple.subject, triple.predicate, triple.object
        );
    }
    Ok(())
}
