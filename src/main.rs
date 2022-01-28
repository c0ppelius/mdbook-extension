use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;
use std::process;
// use regex::{CaptureMatches, Captures, Regex};

fn make_app() -> App<'static, 'static> {
    App::new("mathenvirons")
        .about("A mdbook preprocessor to add latex-like math environments to mdbook.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor")
        )
}

fn main() {
    let matches = make_app().get_matches();

    let preprocessor = MathEvns::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

/// Pre-processor starter, taken straight out of the mdbook book
fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

/// Check to see if we support the processor (latexy only supports html right now),
/// taken straight out of the mdbook book
fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);

    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

pub struct MathEvns;

impl MathEvns {
    pub fn new() -> MathEvns {
        MathEvns
    }
}

impl Preprocessor for MathEvns {
    fn name(&self) -> &str { "mathevirons" }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let mdbook::BookItem::Chapter(chapter) = item {
                let new_contents = replace_placeholders(chapter.content.clone());
                let content = match new_contents {
                    Ok(content) => content,
                    Err(error) => error.to_string()
                };
                chapter.content = content;
            }
        });

        Ok(book)
    }
    
    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

// embarassingly basic string replacements
fn replace_placeholders(content: String) -> Result<String,Error> {
    let theorem_marker = String::from("{#theorem}");
    let theorem_expanded = String::from("<strong>Theorem.</strong>");
    let proofstart_marker = String::from("{#proof}");
    let proofstart_expanded = String::from(
    "<details markdown=\"block\">
    <summary>
    <b>Proof</b>. (Expand to view)
    </summary> 
    <p>");
    let proofend_marker = String::from("{/proof}");
    let proofend_expanded = String::from(
    "</p> 
    <span style=\"float:right;\"> &#9634; </span>&nbsp;
    </details>");

    let new_content = content.replace(&theorem_marker,&theorem_expanded)
                             .replace(&proofend_marker,&proofend_expanded)
                             .replace(&proofstart_marker,&proofstart_expanded);
    Ok(new_content)
}