use crate::parser::{DocumentSt2, Line, Node, DocumentSt1};

pub fn parse(doc: DocumentSt1) -> Result<DocumentSt2, String> {
    let mut nodes = Vec::new();

    // Stack with the current "hierarchy" of nodes being processed.
    //
    // stack.lest() is immediate parent of the lines with stack.len() indent.
    let mut stack: Vec<Node> = Vec::new();

    fn line_to_node(line: Line) -> Node {
        Node {
            contents: line.terms,
            children: Vec::new(),
            bottom_spacing: false,
        }
    }

    fn pop_to_parent(nodes: &mut Vec<Node>, stack: &mut Vec<Node>) {
        let top = stack.pop().expect("stack is empty");

        match stack.last_mut() {
            Some(x) => x.children.push(top),
            None => nodes.push(top),
        }
    }

    for (i, line) in doc.lines.into_iter().enumerate() {
        // In this context, stack.len() corresponds to the indent a line has to be a child of the
        // "current node".

        if line.terms.is_empty() {
            // Empty lines don't result in nodes, but they affect the previous node.

            if let Some(x) = stack.last_mut() {
                x.bottom_spacing = true;
            }
            continue;
        }

        if stack.len() == 0 {
            if line.indent == 0 {
                stack.push(line_to_node(line));
            } else {
                return Err(format!("line {}: indented line before any non-indented line", i + 1));
            }
        } else {
            // pop from the stack if needed
            while line.indent + 1 < stack.len() {
                pop_to_parent(&mut nodes, &mut stack);
            }

            if line.indent + 1 == stack.len() {
                // the current line has the same indent as the last one
                //
                // pop the last line off the stack to its parent, and put the current line on the
                // top of the stack.
                pop_to_parent(&mut nodes, &mut stack);
                stack.push(line_to_node(line));
            } else if line.indent == stack.len() {
                // the current line is a child of the last one
                //
                // push the current line onto the stack
                stack.push(line_to_node(line));
            } else {
                return Err(format!(
                    "line {}: indent leap (current {}, expected at most {})",
                    i + 1,
                    line.indent,
                    stack.len()
                ));
            }
        }
    }

    while !stack.is_empty() {
        pop_to_parent(&mut nodes, &mut stack);
    }

    Ok(DocumentSt2 {
        header: doc.header,
        options: doc.options,
        nodes: nodes,
    })
}
