// Copyright 2020 The FuseQuery Authors.
//
// Code is licensed under Apache License, Version 2.0.

// Borrow from datafusion/logical_plan/display.rs
// See NOTICE.md

use std::fmt;

use crate::planners::PlanNode;
use arrow::datatypes::Schema;

/// Trait that implements the [Visitor
/// pattern](https://en.wikipedia.org/wiki/Visitor_pattern) for a
/// depth first walk of `LogicalPlan` nodes. `pre_visit` is called
/// before any children are visited, and then `post_visit` is called
/// after all children have been visited.
////
/// To use, define a struct that implements this trait and then invoke
/// "LogicalPlan::accept".
///
/// For example, for a logical plan like:
///
/// Projection: #id
///    Filter: #state Eq Utf8(\"CO\")\
///       CsvScan: employee.csv projection=Some([0, 3])";
///
/// The sequence of visit operations would be:
/// ```text
/// visitor.pre_visit(Projection)
/// visitor.pre_visit(Filter)
/// visitor.pre_visit(CsvScan)
/// visitor.post_visit(CsvScan)
/// visitor.post_visit(Filter)
/// visitor.post_visit(Projection)
/// ```
pub trait PlanVisitor {
    /// The type of error returned by this visitor
    type Error;

    /// Invoked on a logical plan before any of its child inputs have been
    /// visited. If Ok(true) is returned, the recursion continues. If
    /// Err(..) or Ok(false) are returned, the recursion stops
    /// immediately and the error, if any, is returned to `accept`
    fn pre_visit(&mut self, plan: &PlanNode) -> std::result::Result<bool, Self::Error>;

    /// Invoked on a logical plan after all of its child inputs have
    /// been visited. The return value is handled the same as the
    /// return value of `pre_visit`. The provided default implementation
    /// returns `Ok(true)`.
    fn post_visit(&mut self, _plan: &PlanNode) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }
}

/// Formats plans with a single line per node. For example:
///
/// Projection: #id
///    Filter: #state Eq Utf8(\"CO\")\
///       CsvScan: employee.csv projection=Some([0, 3])";
pub struct IndentVisitor<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
    /// If true, includes summarized schema information
    with_schema: bool,
    indent: u32,
}

impl<'a, 'b> IndentVisitor<'a, 'b> {
    /// Create a visitor that will write a formatted LogicalPlan to f. If `with_schema` is
    /// true, includes schema information on each line.
    pub fn new(f: &'a mut fmt::Formatter<'b>, with_schema: bool) -> Self {
        Self {
            f,
            with_schema,
            indent: 0,
        }
    }

    fn write_indent(&mut self) -> fmt::Result {
        for _ in 0..self.indent {
            write!(self.f, "  ")?;
        }
        Ok(())
    }
}

impl<'a, 'b> PlanVisitor for IndentVisitor<'a, 'b> {
    type Error = fmt::Error;

    fn pre_visit(&mut self, plan: &PlanNode) -> std::result::Result<bool, fmt::Error> {
        if self.indent > 0 {
            writeln!(self.f)?;
        }
        self.write_indent()?;

        write!(self.f, "{}", plan.display())?;
        if self.with_schema {
            write!(self.f, " {}", display_schema(&plan.schema().as_ref()))?;
        }

        self.indent += 1;
        Ok(true)
    }

    fn post_visit(&mut self, _plan: &PlanNode) -> std::result::Result<bool, fmt::Error> {
        self.indent -= 1;
        Ok(true)
    }
}

pub fn display_schema(schema: &Schema) -> impl fmt::Display + '_ {
    struct Wrapper<'a>(&'a Schema);

    impl<'a> fmt::Display for Wrapper<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "[")?;
            for (idx, field) in self.0.fields().iter().enumerate() {
                if idx > 0 {
                    write!(f, ", ")?;
                }
                let nullable_str = if field.is_nullable() { ";N" } else { "" };
                write!(
                    f,
                    "{}:{:?}{}",
                    field.name(),
                    field.data_type(),
                    nullable_str
                )?;
            }
            write!(f, "]")
        }
    }
    Wrapper(schema)
}

/// Logic related to creating DOT language graphs.
#[derive(Default)]
struct GraphvizBuilder {
    id_gen: usize,
}

impl GraphvizBuilder {
    fn next_id(&mut self) -> usize {
        self.id_gen += 1;
        self.id_gen
    }

    // write out the start of the subgraph cluster
    fn start_cluster(&mut self, f: &mut fmt::Formatter, title: &str) -> fmt::Result {
        writeln!(f, "  subgraph cluster_{}", self.next_id())?;
        writeln!(f, "  {{")?;
        writeln!(f, "    graph[label={}]", Self::quoted(title))
    }

    // write out the end of the subgraph cluster
    fn end_cluster(&mut self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "  }}")
    }

    /// makes a quoted string suitable for inclusion in a graphviz chart
    fn quoted(label: &str) -> String {
        let label = label.replace('"', "_");
        format!("\"{}\"", label)
    }
}

/// Formats plans for graphical display using the `DOT` language. This
/// format can be visualized using software from
/// [`graphviz`](https://graphviz.org/)
pub struct GraphvizVisitor<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
    graphviz_builder: GraphvizBuilder,
    /// If true, includes summarized schema information
    with_schema: bool,

    /// Holds the ids (as generated from `graphviz_builder` of all
    /// parent nodes
    parent_ids: Vec<usize>,
}

impl<'a, 'b> GraphvizVisitor<'a, 'b> {
    pub fn new(f: &'a mut fmt::Formatter<'b>) -> Self {
        Self {
            f,
            graphviz_builder: GraphvizBuilder::default(),
            with_schema: false,
            parent_ids: Vec::new(),
        }
    }

    /// Sets a flag which controls if the output schema is displayed
    pub fn set_with_schema(&mut self, with_schema: bool) {
        self.with_schema = with_schema;
    }

    pub fn pre_visit_plan(&mut self, label: &str) -> fmt::Result {
        self.graphviz_builder.start_cluster(self.f, label)
    }

    pub fn post_visit_plan(&mut self) -> fmt::Result {
        self.graphviz_builder.end_cluster(self.f)
    }
}

impl<'a, 'b> PlanVisitor for GraphvizVisitor<'a, 'b> {
    type Error = fmt::Error;

    fn pre_visit(&mut self, plan: &PlanNode) -> std::result::Result<bool, fmt::Error> {
        let id = self.graphviz_builder.next_id();

        // Create a new graph node for `plan` such as
        // id [label="foo"]
        let label = if self.with_schema {
            format!(
                "{}\\nSchema: {}",
                plan.display(),
                display_schema(&plan.schema().as_ref())
            )
        } else {
            format!("{}", plan.display())
        };

        writeln!(
            self.f,
            "    {}[shape=box label={}]",
            id,
            GraphvizBuilder::quoted(&label)
        )?;

        // Create an edge to our parent node, if any
        //  parent_id -> id
        if let Some(parent_id) = self.parent_ids.last() {
            writeln!(
                self.f,
                "    {} -> {} [arrowhead=none, arrowtail=normal, dir=back]",
                parent_id, id
            )?;
        }

        self.parent_ids.push(id);
        Ok(true)
    }

    fn post_visit(&mut self, _plan: &PlanNode) -> std::result::Result<bool, fmt::Error> {
        // always be non-empty as pre_visit always pushes
        self.parent_ids.pop().unwrap();
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use arrow::datatypes::{DataType, Field};

    use super::*;

    #[test]
    fn test_display_empty_schema() {
        let schema = Schema::new(vec![]);
        assert_eq!("[]", format!("{}", display_schema(&schema)));
    }

    #[test]
    fn test_display_schema() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("first_name", DataType::Utf8, true),
        ]);

        assert_eq!(
            "[id:Int32, first_name:Utf8;N]",
            format!("{}", display_schema(&schema))
        );
    }
}
