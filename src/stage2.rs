use crate::tree::{Document, Line, Node, PreDocument};

pub fn process(doc: PreDocument) -> Result<Document, String> {
    let mut nodes = Vec::new();

    // Stack with the current "hierarchy" of nodes being processed.
    //
    // stack.lest() is immediate parent of the lines with stack.len() indent.
    let mut stack = Vec::new();

    fn line_to_node(line: Line) -> Node {
        Node {
            contents: line.terms,
            children: Vec::new(),
        }
    }

    fn pop(nodes: &mut Vec<Node>, stack: &mut Vec<Node>) {
        let top = stack.pop().expect("stack is empty");

        match stack.last_mut() {
            Some(x) => x.children.push(top),
            None => nodes.push(top),
        }
    }

    for (i, line) in doc.lines.into_iter().enumerate() {
        // pop from the stack if needed
        while line.indent < stack.len() {
            pop(&mut nodes, &mut stack);
        }

        let cur_indent = stack.len();

        if line.indent == cur_indent + 1 {
            stack.push(line_to_node(line));
        } else if line.indent > cur_indent + 1 {
            return Err(format!(
                "line {}: indent leap (current {}, expected at most {})",
                i + 1,
                line.indent,
                cur_indent
            ));
        } else if line.indent == cur_indent {
            // pop the current element from the stack (if any) and put it on its parent, then put
            // the current line on the place where it was on the stack
            if !stack.is_empty() {
                pop(&mut nodes, &mut stack);
            }
            stack.push(line_to_node(line));
        } else {
            unreachable!();
        }
    }

    Ok(Document {
        header: doc.header,
        options: doc.options,
        nodes: nodes,
    })
}
