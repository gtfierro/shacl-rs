use clap::{Parser, ValueEnum};
use env_logger;
use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::exec_dot;
use oxigraph::io::RdfFormat;
use shacl::{Source, Validator};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
#[clap(group(
    clap::ArgGroup::new("shapes_source")
        .required(true)
        .args(&["shapes_file", "shapes_graph"]),
))]
struct ShapesSourceCli {
    /// Path to the shapes file
    #[arg(short, long, value_name = "FILE")]
    shapes_file: Option<PathBuf>,

    /// URI of the shapes graph
    #[arg(long, value_name = "URI")]
    shapes_graph: Option<String>,
}

#[derive(Parser, Debug)]
#[clap(group(
    clap::ArgGroup::new("data_source")
        .required(true)
        .args(&["data_file", "data_graph"]),
))]
struct DataSourceCli {
    /// Path to the data file
    #[arg(short, long, value_name = "FILE")]
    data_file: Option<PathBuf>,

    /// URI of the data graph
    #[arg(long, value_name = "URI")]
    data_graph: Option<String>,
}

#[derive(Parser, Debug)]
struct CommonArgs {
    #[clap(flatten)]
    shapes: ShapesSourceCli,
    #[clap(flatten)]
    data: DataSourceCli,
}

#[derive(Parser)]
struct GraphvizArgs {
    #[clap(flatten)]
    common: CommonArgs,
}

#[derive(Parser)]
struct PdfArgs {
    #[clap(flatten)]
    common: CommonArgs,

    /// Path to the output PDF file
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum ValidateOutputFormat {
    #[default]
    Turtle,
    Dump,
    RdfXml,
    NTriples,
}

#[derive(Parser)]
struct ValidateArgs {
    #[clap(flatten)]
    common: CommonArgs,

    /// The output format for the validation report
    #[arg(long, value_enum, default_value_t = ValidateOutputFormat::Turtle)]
    format: ValidateOutputFormat,
}

#[derive(Parser)]
struct HeatArgs {
    #[clap(flatten)]
    common: CommonArgs,
}

#[derive(Parser)]
struct GraphvizHeatmapArgs {
    #[clap(flatten)]
    common: CommonArgs,

    /// Include all shapes and components, even those not executed
    #[arg(long)]
    all: bool,
}

#[derive(Parser)]
struct PdfHeatmapArgs {
    #[clap(flatten)]
    common: CommonArgs,

    /// Path to the output PDF file
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    /// Include all shapes and components, even those not executed
    #[arg(long)]
    all: bool,
}

#[derive(Parser)]
struct TraceArgs {
    #[clap(flatten)]
    common: CommonArgs,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Output the Graphviz DOT string of the shape graph
    Graphviz(GraphvizArgs),
    /// Generate a PDF of the shape graph using Graphviz
    Pdf(PdfArgs),
    /// Validate the data against the shapes and output a frequency table of component invocations
    Heat(HeatArgs),
    /// Validate the data and output a graphviz heatmap of the shape graph
    #[command(name = "graphviz-heatmap")]
    GraphvizHeatmap(GraphvizHeatmapArgs),
    /// Generate a PDF of the shape graph heatmap using Graphviz
    #[command(name = "pdf-heatmap")]
    PdfHeatmap(PdfHeatmapArgs),
    /// Validate the data against the shapes
    Validate(ValidateArgs),
    /// Print the execution traces for debugging
    Trace(TraceArgs),
}

fn get_validator(common: &CommonArgs) -> Result<Validator, Box<dyn std::error::Error>> {
    let shapes_source = if let Some(path) = &common.shapes.shapes_file {
        Source::File(path.clone())
    } else {
        Source::Graph(common.shapes.shapes_graph.clone().unwrap())
    };

    let data_source = if let Some(path) = &common.data.data_file {
        Source::File(path.clone())
    } else {
        Source::Graph(common.data.data_graph.clone().unwrap())
    };

    Validator::from_sources(shapes_source, data_source)
        .map_err(|e| format!("Error creating validator: {}", e).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Graphviz(args) => {
            let validator = get_validator(&args.common)?;
            let dot_string = validator.to_graphviz()?;
            println!("{}", dot_string);
        }
        Commands::Pdf(args) => {
            let validator = get_validator(&args.common)?;
            let dot_string = validator.to_graphviz()?;

            let output_format = Format::Pdf;
            let output_file_path_str = args
                .output_file
                .to_str()
                .ok_or("Invalid output file path")?;

            let cmd_args = vec![
                CommandArg::Format(output_format),
                CommandArg::Output(output_file_path_str.to_string()),
            ];

            exec_dot(dot_string, cmd_args)
                .map_err(|e| format!("Graphviz execution error: {}", e))?;

            println!("PDF generated at: {}", args.output_file.display());
        }
        Commands::Validate(args) => {
            let validator = get_validator(&args.common)?;
            let report = validator.validate();

            match args.format {
                ValidateOutputFormat::Turtle => {
                    let report_str = report.to_turtle()?;
                    println!("{}", report_str);
                }
                ValidateOutputFormat::Dump => {
                    report.dump();
                }
                ValidateOutputFormat::RdfXml => {
                    let report_str = report.to_rdf(RdfFormat::RdfXml)?;
                    println!("{}", report_str);
                }
                ValidateOutputFormat::NTriples => {
                    let report_str = report.to_rdf(RdfFormat::NTriples)?;
                    println!("{}", report_str);
                }
            }
        }
        Commands::Heat(args) => {
            let validator = get_validator(&args.common)?;
            let report = validator.validate();

            let frequencies: HashMap<(String, String, String), usize> =
                report.get_component_frequencies();

            let mut sorted_frequencies: Vec<_> = frequencies.into_iter().collect();
            sorted_frequencies.sort_by(|a, b| b.1.cmp(&a.1));

            println!("ID\tLabel\tType\tInvocations");
            for ((id, label, item_type), count) in sorted_frequencies {
                println!("{}\t{}\t{}\t{}", id, label, item_type, count);
            }
        }
        Commands::GraphvizHeatmap(args) => {
            let validator = get_validator(&args.common)?;
            // Run validation first to populate execution traces used by graphviz_heatmap.
            // include_all_nodes == args.all: when true, include shapes/components that did not execute.
            let _report = validator.validate();

            let dot_string = validator.to_graphviz_heatmap(args.all)?;
            println!("{}", dot_string);
        }
        Commands::PdfHeatmap(args) => {
            let validator = get_validator(&args.common)?;
            // Run validation first to populate execution traces used by graphviz_heatmap.
            // include_all_nodes == args.all: when true, include shapes/components that did not execute.
            let _report = validator.validate();

            let dot_string = validator.to_graphviz_heatmap(args.all)?;

            let output_format = Format::Pdf;
            let output_file_path_str = args
                .output_file
                .to_str()
                .ok_or("Invalid output file path")?;

            let cmd_args = vec![
                CommandArg::Format(output_format),
                CommandArg::Output(output_file_path_str.to_string()),
            ];

            exec_dot(dot_string, cmd_args)
                .map_err(|e| format!("Graphviz execution error: {}", e))?;

            println!("PDF heatmap generated at: {}", args.output_file.display());
        }
        Commands::Trace(args) => {
            let validator = get_validator(&args.common)?;
            // Run validation to populate execution traces
            let report = validator.validate();

            report.print_traces();
        }
    }
    Ok(())
}
