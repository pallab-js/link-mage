use std::collections::HashMap;
use std::fs;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;

use quick_xml::Writer as XmlWriter;
use quick_xml::events::{BytesDecl, BytesText, Event};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::provenance::Provenance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub properties: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Node {
    pub fn new(node_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node_type: node_type.into(),
            properties: HashMap::new(),
            provenance: None,
        }
    }

    pub fn with_id(id: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.into(),
            properties: HashMap::new(),
            provenance: None,
        }
    }

    pub fn with_provenance(
        node_type: impl Into<String>,
        provenance: Provenance,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node_type: node_type.into(),
            properties: HashMap::new(),
            provenance: Some(provenance),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub properties: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Edge {
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            target: target.into(),
            label: label.into(),
            properties: HashMap::new(),
            provenance: None,
        }
    }

    pub fn with_id(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            label: label.into(),
            properties: HashMap::new(),
            provenance: None,
        }
    }
}

/// The main graph container.
///
/// Both `nodes` and `edges` are backed by `Vec` for cache-friendly iteration
/// and by a `HashMap` index for O(1) id lookups.  The index fields are skipped
/// during (de)serialisation; call [`Graph::rebuild_index`] after constructing a
/// graph from raw data (e.g. `serde_json::from_reader`).
///
/// Removal uses `swap_remove` (O(1)) instead of `remove` (O(n)), so Vec order
/// is not preserved after a deletion – this is intentional and safe because the
/// graph is always accessed by id, never by position.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// id → position in `self.nodes`
    #[serde(skip, default)]
    node_index: HashMap<String, usize>,
    /// id → position in `self.edges`
    #[serde(skip, default)]
    edge_index: HashMap<String, usize>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node. Returns `false` (no-op) if a node with that id already exists.
    /// O(1) amortised.
    pub fn add_node(&mut self, node: Node) -> bool {
        if self.node_index.contains_key(&node.id) {
            return false;
        }
        let idx = self.nodes.len();
        self.node_index.insert(node.id.clone(), idx);
        self.nodes.push(node);
        true
    }

    /// Remove a node and all edges that reference it.  Returns the removed node.
    ///
    /// Uses `swap_remove` (O(1)) for the node itself and for each incident
    /// edge, updating only the affected index entries rather than rebuilding
    /// the whole edge index.  Total cost is O(k) where k = number of incident
    /// edges, instead of O(E) for a full rebuild.
    pub fn remove_node(&mut self, id: &str) -> Option<Node> {
        let &idx = self.node_index.get(id)?;

        // ── Remove the node ───────────────────────────────────────────────────
        let node = self.nodes.swap_remove(idx);
        self.node_index.remove(id);
        if idx < self.nodes.len() {
            self.node_index.insert(self.nodes[idx].id.clone(), idx);
        }

        // ── Cascade-remove incident edges (targeted swap_remove) ───────────────
        // Walk the Vec from the back so swap_remove never skips an element that
        // was just moved into the current position.
        let mut i = self.edges.len();
        while i > 0 {
            i -= 1;
            if self.edges[i].source == id || self.edges[i].target == id {
                let eid = self.edges[i].id.clone();
                self.edge_index.remove(&eid);
                self.edges.swap_remove(i);
                // The previously-last edge is now at position i — fix its index.
                if i < self.edges.len() {
                    self.edge_index.insert(self.edges[i].id.clone(), i);
                }
            }
        }

        Some(node)
    }

    /// Add an edge. Returns `false` if source/target are missing or the id is
    /// already present. All checks are O(1).
    pub fn add_edge(&mut self, edge: Edge) -> bool {
        if !self.node_index.contains_key(&edge.source)
            || !self.node_index.contains_key(&edge.target)
            || self.edge_index.contains_key(&edge.id)
        {
            return false;
        }
        let idx = self.edges.len();
        self.edge_index.insert(edge.id.clone(), idx);
        self.edges.push(edge);
        true
    }

    /// Remove an edge by id.  Returns the removed edge.  O(1) amortised.
    pub fn remove_edge(&mut self, id: &str) -> Option<Edge> {
        let &idx = self.edge_index.get(id)?;
        let edge = self.edges.swap_remove(idx);
        self.edge_index.remove(id);
        if idx < self.edges.len() {
            self.edge_index.insert(self.edges[idx].id.clone(), idx);
        }
        Some(edge)
    }

    /// Find a node by id (immutable borrow).  O(1).
    pub fn find_node(&self, id: &str) -> Option<&Node> {
        let &idx = self.node_index.get(id)?;
        self.nodes.get(idx)
    }

    /// Find a node by id (mutable borrow).  O(1).
    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        let &idx = self.node_index.get(id)?;
        self.nodes.get_mut(idx)
    }

    /// Rebuild both id→index maps from the current `nodes`/`edges` Vecs.
    ///
    /// Call this after constructing a `Graph` by deserialising raw JSON
    /// (where the index fields are skipped by serde).
    pub fn rebuild_index(&mut self) {
        self.node_index = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), i))
            .collect();
        self.edge_index = self
            .edges
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect();
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    /// Serialize the graph to pretty-printed JSON and write it to `path`.
    pub fn save_graph(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    /// Deserialize a `Graph` from a JSON file at `path`.
    ///
    /// Uses a buffered reader to avoid reading the whole file into a `String`
    /// first, then rebuilds the id→index maps that serde skips.
    pub fn load_graph(path: &Path) -> io::Result<Graph> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut g: Graph = serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        g.rebuild_index();
        Ok(g)
    }

    // ── CSV export ─────────────────────────────────────────────────────────────

    /// Write two CSV files into a directory:
    ///
    /// * `nodes_path` — columns: `id`, `type`, `properties` (JSON string)
    /// * `edges_path` — columns: `id`, `source`, `target`, `label`, `properties` (JSON string)
    ///
    /// Existing files at those paths are overwritten.  Parent directories are
    /// created if they do not exist.
    pub fn export_csv(&self, nodes_path: &Path, edges_path: &Path) -> io::Result<()> {
        // Nodes
        if let Some(parent) = nodes_path.parent() {
            fs::create_dir_all(parent)?;
        }
        {
            let file = fs::File::create(nodes_path)?;
            let mut w = BufWriter::new(file);
            w.write_all(b"id,type,properties\n")?;
            for node in &self.nodes {
                let props_json = serde_json::to_string(&node.properties)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                writeln!(
                    w,
                    "{},{},{}",
                    csv_field(&node.id),
                    csv_field(&node.node_type),
                    csv_field(&props_json),
                )?;
            }
        }

        // Edges
        if let Some(parent) = edges_path.parent() {
            fs::create_dir_all(parent)?;
        }
        {
            let file = fs::File::create(edges_path)?;
            let mut w = BufWriter::new(file);
            w.write_all(b"id,source,target,label,properties\n")?;
            for edge in &self.edges {
                let props_json = serde_json::to_string(&edge.properties)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                writeln!(
                    w,
                    "{},{},{},{},{}",
                    csv_field(&edge.id),
                    csv_field(&edge.source),
                    csv_field(&edge.target),
                    csv_field(&edge.label),
                    csv_field(&props_json),
                )?;
            }
        }

        Ok(())
    }

    // ── CSV import ─────────────────────────────────────────────────────────────

    /// Reconstruct a [`Graph`] from two CSV files previously written by
    /// [`export_csv`](Graph::export_csv).
    ///
    /// **Every element receives a fresh UUID.**  The original IDs in the CSV
    /// are used only to wire up edge `source`/`target` references; they are
    /// never re-used as the returned node/edge IDs.  This prevents collisions
    /// when merging an imported graph into an existing one.
    ///
    /// # CSV format
    /// * `nodes_path` — header `id,type,properties` where `properties` is a
    ///   compact JSON object string.
    /// * `edges_path` — header `id,source,target,label,properties` where
    ///   `source`/`target` are the node IDs from the **same** CSV pair.
    ///
    /// # Errors
    /// Returns [`io::Error`] with kind [`InvalidData`](io::ErrorKind::InvalidData)
    /// if:
    /// * either file cannot be read,
    /// * a row has fewer columns than expected,
    /// * a `properties` cell is not valid JSON, or
    /// * an edge references a node ID that was not present in `nodes_path`.
    pub fn import_csv(nodes_path: &Path, edges_path: &Path) -> io::Result<Graph> {
        let mut graph = Graph::new();
        // original CSV id → new UUID assigned on import
        let mut id_map: HashMap<String, String> = HashMap::new();

        // ── Nodes ─────────────────────────────────────────────────────────────
        let nodes_text = fs::read_to_string(nodes_path)?;
        let mut node_lines = nodes_text.lines();

        // Validate header
        let node_header = node_lines.next().ok_or_else(|| {
            io_invalid("nodes CSV is empty")
        })?;
        let node_cols = parse_csv_row(node_header);
        if node_cols.len() < 3 {
            return Err(io_invalid(format!(
                "nodes CSV header has {} column(s), expected ≥ 3 (id, type, properties)",
                node_cols.len()
            )));
        }

        for (i, line) in node_lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let row_num = i + 2; // 1-based, accounting for header
            let fields = parse_csv_row(line);
            if fields.len() < 3 {
                return Err(io_invalid(format!(
                    "nodes CSV row {row_num}: expected ≥ 3 columns, got {}",
                    fields.len()
                )));
            }

            let original_id = fields[0].clone();
            let node_type   = fields[1].clone();
            let properties  = json_to_str_map(&fields[2], row_num, "nodes")?;

            let new_id = Uuid::new_v4().to_string();
            id_map.insert(original_id, new_id.clone());
            graph.add_node(Node { id: new_id, node_type, properties, provenance: None });
        }

        // ── Edges ─────────────────────────────────────────────────────────────
        let edges_text = fs::read_to_string(edges_path)?;
        let mut edge_lines = edges_text.lines();

        // Validate header
        let edge_header = edge_lines.next().ok_or_else(|| {
            io_invalid("edges CSV is empty")
        })?;
        let edge_cols = parse_csv_row(edge_header);
        if edge_cols.len() < 5 {
            return Err(io_invalid(format!(
                "edges CSV header has {} column(s), expected ≥ 5 \
                 (id, source, target, label, properties)",
                edge_cols.len()
            )));
        }

        for (i, line) in edge_lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let row_num = i + 2;
            let fields = parse_csv_row(line);
            if fields.len() < 5 {
                return Err(io_invalid(format!(
                    "edges CSV row {row_num}: expected ≥ 5 columns, got {}",
                    fields.len()
                )));
            }

            // fields[0] = original edge id (discarded — new UUID assigned below)
            let orig_source = &fields[1];
            let orig_target = &fields[2];
            let label       = fields[3].clone();
            let properties  = json_to_str_map(&fields[4], row_num, "edges")?;

            let new_source = id_map.get(orig_source.as_str()).ok_or_else(|| {
                io_invalid(format!(
                    "edges CSV row {row_num}: source node '{orig_source}' \
                     not found in nodes CSV"
                ))
            })?;
            let new_target = id_map.get(orig_target.as_str()).ok_or_else(|| {
                io_invalid(format!(
                    "edges CSV row {row_num}: target node '{orig_target}' \
                     not found in nodes CSV"
                ))
            })?;

            // add_edge validates source/target existence — should always succeed here.
            graph.add_edge(Edge {
                id:     Uuid::new_v4().to_string(),
                source: new_source.clone(),
                target: new_target.clone(),
                label,
                properties,
                provenance: None,
            });
        }

        Ok(graph)
    }

    // ── GraphML export ─────────────────────────────────────────────────────────

    /// Serialize the graph to a GraphML 1.0 file at `path`.
    ///
    /// The generated file is valid GraphML that can be opened by Gephi, yEd,
    /// Cytoscape, NetworkX, and other graph tools.
    ///
    /// ## Schema
    ///
    /// ### Node `<key>` declarations
    /// | id | attr.name | description |
    /// |----|-----------|-------------|
    /// | `d_type` | `type` | node type string (e.g. "IP", "Domain") |
    /// | `d_label` | `label` | display label (from `properties["label"]`, fallback to id) |
    /// | `d_props` | `properties` | full properties map as JSON string |
    ///
    /// ### Edge `<key>` declarations
    /// | id | attr.name | description |
    /// |----|-----------|-------------|
    /// | `d_edge_label` | `label` | edge relationship type |
    /// | `d_edge_props` | `properties` | full properties map as JSON string |
    pub fn export_graphml(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::File::create(path)?;
        let mut w = XmlWriter::new_with_indent(BufWriter::new(file), b' ', 2);

        // <?xml version="1.0" encoding="UTF-8"?>
        w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        // <graphml xmlns=… xmlns:xsi=… xsi:schemaLocation=…>
        w.create_element("graphml")
            .with_attribute(("xmlns",
                "http://graphml.graphdrawing.org/graphml"))
            .with_attribute(("xmlns:xsi",
                "http://www.w3.org/2001/XMLSchema-instance"))
            .with_attribute(("xsi:schemaLocation",
                "http://graphml.graphdrawing.org/graphml \
                 http://graphml.graphdrawing.org/graphml/1.0/graphml.xsd"))
            .write_inner_content(|w| {
                // ── Key declarations ─────────────────────────────────────
                graphml_key(w, "d_type",       "node", "type",       "string")?;
                graphml_key(w, "d_label",      "node", "label",      "string")?;
                graphml_key(w, "d_props",      "node", "properties", "string")?;
                graphml_key(w, "d_edge_label", "edge", "label",      "string")?;
                graphml_key(w, "d_edge_props", "edge", "properties", "string")?;

                // ── <graph id="G" edgedefault="directed"> ─────────────────
                w.create_element("graph")
                    .with_attribute(("id", "G"))
                    .with_attribute(("edgedefault", "directed"))
                    .write_inner_content(|w| {
                        // Nodes
                        for node in &self.nodes {
                            let label = node.properties
                                .get("label")
                                .or_else(|| node.properties.get("value"))
                                .map(String::as_str)
                                .unwrap_or(node.id.as_str());

                            let props_json = serde_json::to_string(&node.properties)
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                            w.create_element("node")
                                .with_attribute(("id", node.id.as_str()))
                                .write_inner_content(|w| {
                                    graphml_data(w, "d_type",  &node.node_type)?;
                                    graphml_data(w, "d_label", label)?;
                                    graphml_data(w, "d_props", &props_json)?;
                                    Ok(())
                                })?;
                        }

                        // Edges
                        for edge in &self.edges {
                            let props_json = serde_json::to_string(&edge.properties)
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                            w.create_element("edge")
                                .with_attribute(("id",     edge.id.as_str()))
                                .with_attribute(("source", edge.source.as_str()))
                                .with_attribute(("target", edge.target.as_str()))
                                .write_inner_content(|w| {
                                    graphml_data(w, "d_edge_label", &edge.label)?;
                                    graphml_data(w, "d_edge_props", &props_json)?;
                                    Ok(())
                                })?;
                        }

                        Ok(())
                    })?;

                Ok(())
            })?;

        Ok(())
    }

    /// Replaces a set of nodes with a single "Collection" node.
    /// All edges incident to the original nodes are re-routed to the collection node.
    /// Returns the newly created collection node.
    pub fn collapse_nodes(&mut self, ids: &[String], label: &str) -> Result<Node, String> {
        if ids.is_empty() {
            return Err("No nodes selected to collapse".into());
        }

        // Create the collection node
        let mut collection = Node::new("Collection");
        collection.properties.insert("label".into(), label.to_string());
        collection.properties.insert("value".into(), label.to_string());
        collection.properties.insert("count".into(), ids.len().to_string());
        
        let col_id = collection.id.clone();
        let id_set: std::collections::HashSet<String> = ids.iter().cloned().collect();

        // 1. Add collection node
        self.add_node(collection.clone());

        // 2. Re-route edges
        for edge in self.edges.iter_mut() {
            let src_match = id_set.contains(&edge.source);
            let tgt_match = id_set.contains(&edge.target);
            
            if src_match { edge.source = col_id.clone(); }
            if tgt_match { edge.target = col_id.clone(); }
        }

        // 3. Remove original nodes
        for id in ids {
            self.remove_node(id);
        }

        // 4. Final check: remove any self-loops created by collapsing both ends of an edge
        self.edges.retain(|e| e.source != e.target);
        self.rebuild_index();

        Ok(collection)
    }
}

// ── GraphML helpers ────────────────────────────────────────────────────────────

/// Write a self-closing `<key>` declaration element.
fn graphml_key<W: Write>(
    w: &mut XmlWriter<W>,
    id: &str,
    for_el: &str,
    attr_name: &str,
    attr_type: &str,
) -> io::Result<()> {
    w.create_element("key")
        .with_attribute(("id",        id))
        .with_attribute(("for",       for_el))
        .with_attribute(("attr.name", attr_name))
        .with_attribute(("attr.type", attr_type))
        .write_empty()
        .map(|_| ())
}

/// Write `<data key="KEY">VALUE</data>`, escaping `VALUE` per XML rules.
fn graphml_data<W: Write>(
    w: &mut XmlWriter<W>,
    key: &str,
    value: &str,
) -> io::Result<()> {
    w.create_element("data")
        .with_attribute(("key", key))
        // BytesText::new escapes <, >, &, " automatically.
        // For JSON content that may contain additional special chars, we ensure proper escaping.
        .write_text_content(BytesText::new(value))
        .map(|_| ())
}

// ── CSV helpers ────────────────────────────────────────────────────────────────

/// Shorthand for `io::Error` with kind `InvalidData`.
fn io_invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

/// Deserialise a compact JSON object string into `HashMap<String, String>`.
fn json_to_str_map(
    json: &str,
    row_num: usize,
    file: &str,
) -> io::Result<HashMap<String, String>> {
    serde_json::from_str(json).map_err(|e| {
        io_invalid(format!(
            "{file} CSV row {row_num}: invalid properties JSON — {e}"
        ))
    })
}

/// Parse one line of RFC-4180 CSV into an owned `Vec<String>`.
///
/// * Quoted fields (`"…"`) may contain commas and escaped double-quotes (`""`).
/// * Unquoted fields are delimited by commas.
/// * A trailing comma produces a final empty field.
/// * Windows `\r` is stripped from unquoted field tails.
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let mut chars = line.chars().peekable();

    loop {
        let field = if chars.peek() == Some(&'"') {
            // ── Quoted field ─────────────────────────────────────────────
            chars.next(); // consume opening `"`
            let mut val = String::new();
            loop {
                match chars.next() {
                    Some('"') => {
                        if chars.peek() == Some(&'"') {
                            chars.next(); // `""` → single `"`
                            val.push('"');
                        } else {
                            break; // closing quote
                        }
                    }
                    Some(c) => val.push(c),
                    None => break, // unterminated — treat as closed
                }
            }
            val
        } else {
            // ── Unquoted field ───────────────────────────────────────────
            let mut val = String::new();
            loop {
                match chars.peek().copied() {
                    Some(',') | None => break,
                    Some(c) => {
                        chars.next();
                        val.push(c);
                    }
                }
            }
            // Strip stray Windows CR at end of last field on the line.
            if val.ends_with('\r') {
                val.pop();
            }
            val
        };

        fields.push(field);

        // Consume field separator or stop at EOL.
        match chars.peek().copied() {
            Some(',') => {
                chars.next();
            }
            _ => break,
        }
    }

    fields
}

/// RFC-4180 CSV field escaping with formula injection protection.
///
/// Returns the field as-is if it contains no special characters, otherwise
/// wraps it in double-quotes and escapes any embedded double-quotes by doubling
/// them (`"` → `""`).
///
/// **Security:** To prevent CSV injection attacks (where a cell starting with
/// `=`, `+`, `-`, or `@` could be interpreted as a formula by spreadsheet apps),
/// we prefix such fields with a single quote which forces literal interpretation.
fn csv_field(s: &str) -> String {
    // CSV injection protection: prefix formula-starting characters with '
    let needs_formula_escape = s.starts_with(['=', '+', '-', '@']);
    
    let needs_quote = s.contains([',', '"', '\n', '\r']);
    
    if needs_formula_escape {
        // Prefix with single quote to force literal interpretation in spreadsheets
        format!("'{}", s)
    } else if needs_quote {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> (Graph, String, String) {
        let mut g = Graph::new();
        let a = Node::with_id("node-a", "IP");
        let b = Node::with_id("node-b", "Domain");
        let id_a = a.id.clone();
        let id_b = b.id.clone();
        g.add_node(a);
        g.add_node(b);
        (g, id_a, id_b)
    }

    // ── Node tests ─────────────────────────────────────────────────────────────

    #[test]
    fn add_node_succeeds() {
        let mut g = Graph::new();
        let n = Node::with_id("n1", "IP");
        assert!(g.add_node(n));
        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn add_duplicate_node_is_rejected() {
        let mut g = Graph::new();
        g.add_node(Node::with_id("n1", "IP"));
        let duplicate = Node::with_id("n1", "Domain");
        assert!(!g.add_node(duplicate));
        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn find_node_returns_correct_node() {
        let (g, id_a, _) = make_graph();
        let node = g.find_node(&id_a).expect("node-a should exist");
        assert_eq!(node.id, id_a);
        assert_eq!(node.node_type, "IP");
    }

    #[test]
    fn find_node_returns_none_for_missing() {
        let g = Graph::new();
        assert!(g.find_node("ghost").is_none());
    }

    #[test]
    fn remove_node_returns_node_and_shrinks_list() {
        let (mut g, id_a, _) = make_graph();
        let removed = g.remove_node(&id_a).expect("should remove node-a");
        assert_eq!(removed.id, id_a);
        assert_eq!(g.nodes.len(), 1);
        assert!(g.find_node(&id_a).is_none());
    }

    #[test]
    fn remove_node_cascades_to_edges() {
        let (mut g, id_a, id_b) = make_graph();
        let e = Edge::with_id("e1", &id_a, &id_b, "resolves");
        g.add_edge(e);
        assert_eq!(g.edges.len(), 1);

        g.remove_node(&id_a);
        assert_eq!(g.edges.len(), 0, "edge referencing removed node must be deleted");
    }

    #[test]
    fn remove_nonexistent_node_returns_none() {
        let mut g = Graph::new();
        assert!(g.remove_node("ghost").is_none());
    }

    // ── Edge tests ─────────────────────────────────────────────────────────────

    #[test]
    fn add_edge_succeeds() {
        let (mut g, id_a, id_b) = make_graph();
        let e = Edge::with_id("e1", &id_a, &id_b, "resolves");
        assert!(g.add_edge(e));
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn add_edge_fails_for_missing_source() {
        let (mut g, _, id_b) = make_graph();
        let e = Edge::with_id("e1", "does-not-exist", &id_b, "resolves");
        assert!(!g.add_edge(e));
        assert_eq!(g.edges.len(), 0);
    }

    #[test]
    fn add_edge_fails_for_missing_target() {
        let (mut g, id_a, _) = make_graph();
        let e = Edge::with_id("e1", &id_a, "does-not-exist", "resolves");
        assert!(!g.add_edge(e));
    }

    #[test]
    fn add_duplicate_edge_is_rejected() {
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves"));
        let dup = Edge::with_id("e1", &id_a, &id_b, "other");
        assert!(!g.add_edge(dup));
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn remove_edge_returns_edge_and_shrinks_list() {
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves"));
        let removed = g.remove_edge("e1").expect("should remove e1");
        assert_eq!(removed.id, "e1");
        assert_eq!(g.edges.len(), 0);
    }

    #[test]
    fn remove_nonexistent_edge_returns_none() {
        let mut g = Graph::new();
        assert!(g.remove_edge("ghost").is_none());
    }

    // ── Serde round-trip ───────────────────────────────────────────────────────

    #[test]
    fn graph_serialises_and_deserialises() {
        let (g, _, _) = make_graph();
        let json = serde_json::to_string(&g).expect("serialise");
        let g2: Graph = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(g.nodes.len(), g2.nodes.len());
        assert_eq!(g.edges.len(), g2.edges.len());
    }

    #[test]
    fn node_type_field_serialises_as_type() {
        let n = Node::with_id("n1", "IP");
        let json = serde_json::to_string(&n).expect("serialise");
        assert!(json.contains(r#""type":"IP""#));
        assert!(!json.contains("node_type"));
    }

    #[test]
    fn uuid_generated_ids_are_unique() {
        let n1 = Node::new("IP");
        let n2 = Node::new("IP");
        assert_ne!(n1.id, n2.id);
    }

    // ── File I/O ───────────────────────────────────────────────────────────────

    #[test]
    fn save_and_load_roundtrip() {
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves"));

        let dir = std::env::temp_dir();
        let path = dir.join("link_mage_test_graph.json");

        g.save_graph(&path).expect("save should succeed");
        assert!(path.exists(), "file should be created");

        let loaded = Graph::load_graph(&path).expect("load should succeed");
        assert_eq!(loaded.nodes.len(), g.nodes.len());
        assert_eq!(loaded.edges.len(), g.edges.len());
        assert_eq!(loaded.find_node(&id_a).unwrap().node_type, "IP");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_nonexistent_file_returns_error() {
        let path = std::path::Path::new("/tmp/link_mage_does_not_exist_xyz.json");
        assert!(Graph::load_graph(path).is_err());
    }

    #[test]
    fn load_invalid_json_returns_error() {
        let dir = std::env::temp_dir();
        let path = dir.join("link_mage_bad.json");
        std::fs::write(&path, b"not valid json {{{").unwrap();
        assert!(Graph::load_graph(&path).is_err());
        std::fs::remove_file(&path).ok();
    }

    // ── CSV export ─────────────────────────────────────────────────────────────

    #[test]
    fn csv_field_passthrough_simple() {
        assert_eq!(csv_field("hello"), "hello");
        assert_eq!(csv_field("192.168.1.1"), "192.168.1.1");
    }

    #[test]
    fn csv_field_quotes_commas() {
        assert_eq!(csv_field("a,b"), "\"a,b\"");
    }

    #[test]
    fn csv_field_escapes_embedded_quotes() {
        assert_eq!(csv_field(r#"say "hi""#), r#""say ""hi""""#);
    }

    #[test]
    fn csv_field_quotes_newlines() {
        assert_eq!(csv_field("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn export_csv_creates_files_with_headers() {
        let (mut g, id_a, id_b) = make_graph();
        g.nodes[0]
            .properties
            .insert("value".into(), "10.0.0.1".into());
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves-to"));

        let dir = std::env::temp_dir().join("link_mage_csv_test");
        let nodes_path = dir.join("nodes.csv");
        let edges_path = dir.join("edges.csv");

        g.export_csv(&nodes_path, &edges_path)
            .expect("export should succeed");

        let nodes_csv = std::fs::read_to_string(&nodes_path).unwrap();
        let edges_csv = std::fs::read_to_string(&edges_path).unwrap();

        // Headers present
        assert!(nodes_csv.starts_with("id,type,properties\n"), "wrong nodes header");
        assert!(edges_csv.starts_with("id,source,target,label,properties\n"), "wrong edges header");

        // Correct row counts (header + 2 nodes)
        assert_eq!(nodes_csv.lines().count(), 3, "expected header + 2 node rows");
        // Edge rows
        assert_eq!(edges_csv.lines().count(), 2, "expected header + 1 edge row");

        // Node values present
        assert!(nodes_csv.contains("node-a"));
        assert!(nodes_csv.contains("node-b"));
        assert!(nodes_csv.contains("IP"));
        assert!(nodes_csv.contains("Domain"));

        // Edge values present
        assert!(edges_csv.contains("e1"));
        assert!(edges_csv.contains("resolves-to"));
        assert!(edges_csv.contains(&id_a));
        assert!(edges_csv.contains(&id_b));

        // Cleanup
        std::fs::remove_file(&nodes_path).ok();
        std::fs::remove_file(&edges_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn export_csv_properties_as_json() {
        let mut g = Graph::new();
        let mut n = Node::with_id("n1", "IP");
        n.properties.insert("value".into(), "1.2.3.4".into());
        n.properties
            .insert("note".into(), "has, comma".into());
        g.add_node(n);

        let dir = std::env::temp_dir().join("link_mage_csv_props_test");
        let np = dir.join("nodes.csv");
        let ep = dir.join("edges.csv");
        g.export_csv(&np, &ep).unwrap();

        let csv = std::fs::read_to_string(&np).unwrap();
        // The properties column should be a quoted JSON string.
        // The JSON contains a comma so the whole field must be quoted.
        let row = csv.lines().nth(1).unwrap();
        // properties cell starts with `"` because JSON is quoted
        let fields: Vec<&str> = row.splitn(3, ',').collect();
        assert_eq!(fields[0], "n1");
        assert_eq!(fields[1], "IP");
        assert!(
            fields[2].starts_with('"'),
            "properties JSON must be quoted: {}",
            fields[2]
        );

        std::fs::remove_file(&np).ok();
        std::fs::remove_file(&ep).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn export_csv_empty_graph() {
        let g = Graph::new();
        let dir = std::env::temp_dir().join("link_mage_csv_empty_test");
        let np = dir.join("nodes.csv");
        let ep = dir.join("edges.csv");
        g.export_csv(&np, &ep).unwrap();

        assert_eq!(std::fs::read_to_string(&np).unwrap(), "id,type,properties\n");
        assert_eq!(
            std::fs::read_to_string(&ep).unwrap(),
            "id,source,target,label,properties\n"
        );

        std::fs::remove_file(&np).ok();
        std::fs::remove_file(&ep).ok();
        std::fs::remove_dir(&dir).ok();
    }

    // ── parse_csv_row ──────────────────────────────────────────────────────────

    #[test]
    fn csv_row_simple_three_fields() {
        assert_eq!(parse_csv_row("a,b,c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn csv_row_quoted_field_with_comma() {
        assert_eq!(parse_csv_row(r#"a,"b,c",d"#), vec!["a", "b,c", "d"]);
    }

    #[test]
    fn csv_row_quoted_field_with_escaped_quote() {
        // `""` inside a quoted field → single `"`
        assert_eq!(parse_csv_row(r#""say ""hi""",world"#), vec![r#"say "hi""#, "world"]);
    }

    #[test]
    fn csv_row_empty_unquoted_fields() {
        assert_eq!(parse_csv_row("a,,c"), vec!["a", "", "c"]);
    }

    #[test]
    fn csv_row_trailing_comma_adds_empty_field() {
        assert_eq!(parse_csv_row("a,b,"), vec!["a", "b", ""]);
    }

    #[test]
    fn csv_row_single_field() {
        assert_eq!(parse_csv_row("hello"), vec!["hello"]);
    }

    #[test]
    fn csv_row_empty_quoted_field() {
        assert_eq!(parse_csv_row(r#""","b""#), vec!["", "b"]);
    }

    #[test]
    fn csv_row_strips_windows_cr() {
        // Last field of a Windows-line-ending line may have a trailing \r.
        let line = "a,b,c\r";
        assert_eq!(parse_csv_row(line), vec!["a", "b", "c"]);
    }

    #[test]
    fn csv_row_json_object_as_quoted_field() {
        // Realistic properties column: JSON with commas, quoted by csv_field.
        let json = r#"{"value":"1.2.3.4","note":"a, b"}"#;
        let csv_line = format!("node-id,IP,{}", csv_field(json));
        let fields = parse_csv_row(&csv_line);
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[2], json);
    }

    // ── import_csv ─────────────────────────────────────────────────────────────

    /// Export `g`, then import it back and return the reconstructed graph.
    fn csv_roundtrip(g: &Graph) -> Graph {
        let dir = std::env::temp_dir()
            .join(format!("link_mage_import_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let np = dir.join("nodes.csv");
        let ep = dir.join("edges.csv");
        g.export_csv(&np, &ep).unwrap();
        let imported = Graph::import_csv(&np, &ep).unwrap();
        std::fs::remove_file(&np).ok();
        std::fs::remove_file(&ep).ok();
        std::fs::remove_dir(&dir).ok();
        imported
    }

    #[test]
    fn import_csv_node_count_matches() {
        let (g, _, _) = make_graph();
        let imported = csv_roundtrip(&g);
        assert_eq!(imported.nodes.len(), g.nodes.len());
    }

    #[test]
    fn import_csv_edge_count_matches() {
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves-to"));
        let imported = csv_roundtrip(&g);
        assert_eq!(imported.edges.len(), 1);
    }

    #[test]
    fn import_csv_generates_fresh_uuids() {
        let (g, id_a, id_b) = make_graph();
        let imported = csv_roundtrip(&g);
        // The imported node IDs must NOT equal the original IDs.
        let imported_ids: std::collections::HashSet<&str> =
            imported.nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(!imported_ids.contains(id_a.as_str()), "old ID must not be reused");
        assert!(!imported_ids.contains(id_b.as_str()), "old ID must not be reused");
    }

    #[test]
    fn import_csv_preserves_node_type_and_properties() {
        let mut g = Graph::new();
        let mut n = Node::with_id("original-id", "IP");
        n.properties.insert("value".into(), "10.0.0.1".into());
        n.properties.insert("label".into(), "10.0.0.1".into());
        g.add_node(n);

        let imported = csv_roundtrip(&g);
        let node = &imported.nodes[0];
        assert_eq!(node.node_type, "IP");
        assert_eq!(node.properties["value"], "10.0.0.1");
        assert_ne!(node.id, "original-id", "ID must be fresh UUID");
    }

    #[test]
    fn import_csv_edge_source_target_are_remapped() {
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves-to"));

        let imported = csv_roundtrip(&g);
        let edge = &imported.edges[0];

        // The imported source/target must be valid IDs in the imported graph.
        assert!(
            imported.find_node(&edge.source).is_some(),
            "edge source must reference an imported node"
        );
        assert!(
            imported.find_node(&edge.target).is_some(),
            "edge target must reference an imported node"
        );
        // And they must be different from the originals.
        assert_ne!(edge.source, id_a);
        assert_ne!(edge.target, id_b);
    }

    #[test]
    fn import_csv_edge_label_preserved() {
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves-to"));
        let imported = csv_roundtrip(&g);
        assert_eq!(imported.edges[0].label, "resolves-to");
    }

    #[test]
    fn import_csv_properties_with_special_chars_roundtrip() {
        // Value containing commas and quotes — exercises the CSV quoting path.
        let mut g = Graph::new();
        let mut n = Node::with_id("n1", "Info");
        n.properties.insert("note".into(), r#"a, b "quoted""#.into());
        g.add_node(n);
        let imported = csv_roundtrip(&g);
        assert_eq!(imported.nodes[0].properties["note"], r#"a, b "quoted""#);
    }

    #[test]
    fn import_csv_empty_graph_roundtrip() {
        let g = Graph::new();
        let imported = csv_roundtrip(&g);
        assert!(imported.nodes.is_empty());
        assert!(imported.edges.is_empty());
    }

    #[test]
    fn import_csv_unknown_edge_source_returns_error() {
        let dir = std::env::temp_dir()
            .join(format!("link_mage_err_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let np = dir.join("nodes.csv");
        let ep = dir.join("edges.csv");

        std::fs::write(&np, "id,type,properties\nn1,IP,{}\n").unwrap();
        // source "ghost" doesn't exist in nodes.csv
        std::fs::write(&ep,
            "id,source,target,label,properties\ne1,ghost,n1,link,{}\n"
        ).unwrap();

        let result = Graph::import_csv(&np, &ep);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("ghost"), "error should name the unknown ID: {msg}");

        std::fs::remove_file(&np).ok();
        std::fs::remove_file(&ep).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn import_csv_bad_json_returns_error() {
        let dir = std::env::temp_dir()
            .join(format!("link_mage_badjson_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let np = dir.join("nodes.csv");
        let ep = dir.join("edges.csv");

        std::fs::write(&np, "id,type,properties\nn1,IP,not_json\n").unwrap();
        std::fs::write(&ep, "id,source,target,label,properties\n").unwrap();

        let result = Graph::import_csv(&np, &ep);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("JSON"));

        std::fs::remove_file(&np).ok();
        std::fs::remove_file(&ep).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn import_csv_too_few_node_columns_returns_error() {
        let dir = std::env::temp_dir()
            .join(format!("link_mage_cols_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let np = dir.join("nodes.csv");
        let ep = dir.join("edges.csv");

        // Only 2 columns instead of 3
        std::fs::write(&np, "id,type,properties\nn1,IP\n").unwrap();
        std::fs::write(&ep, "id,source,target,label,properties\n").unwrap();

        let result = Graph::import_csv(&np, &ep);
        assert!(result.is_err());

        std::fs::remove_file(&np).ok();
        std::fs::remove_file(&ep).ok();
        std::fs::remove_dir(&dir).ok();
    }

    // ── GraphML export ─────────────────────────────────────────────────────────

    fn export_graphml_to_string(g: &Graph) -> String {
        // Each call gets a unique path so parallel test runs don't race.
        let path = std::env::temp_dir()
            .join(format!("link_mage_{}.graphml", Uuid::new_v4()));
        g.export_graphml(&path).expect("export should succeed");
        let s = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();
        s
    }

    #[test]
    fn graphml_starts_with_xml_declaration() {
        let g = Graph::new();
        let xml = export_graphml_to_string(&g);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "missing XML declaration: {}", &xml[..xml.len().min(80)]);
    }

    #[test]
    fn graphml_contains_graphml_namespace() {
        let g = Graph::new();
        let xml = export_graphml_to_string(&g);
        assert!(xml.contains("http://graphml.graphdrawing.org/graphml"),
            "missing GraphML namespace");
    }

    #[test]
    fn graphml_key_declarations_present() {
        let g = Graph::new();
        let xml = export_graphml_to_string(&g);
        assert!(xml.contains("id=\"d_type\""),      "missing d_type key");
        assert!(xml.contains("id=\"d_label\""),     "missing d_label key");
        assert!(xml.contains("id=\"d_props\""),     "missing d_props key");
        assert!(xml.contains("id=\"d_edge_label\""),"missing d_edge_label key");
        assert!(xml.contains("id=\"d_edge_props\""),"missing d_edge_props key");
        assert!(xml.contains("for=\"node\""), "missing for=node");
        assert!(xml.contains("for=\"edge\""), "missing for=edge");
    }

    #[test]
    fn graphml_contains_node_and_edge_data() {
        let (mut g, id_a, id_b) = make_graph();
        g.nodes[0].properties.insert("value".into(), "10.0.0.1".into());
        g.nodes[0].properties.insert("label".into(), "10.0.0.1".into());
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "resolves-to"));

        let xml = export_graphml_to_string(&g);

        // Node elements
        assert!(xml.contains(&format!("<node id=\"{}\"", id_a)), "missing node-a element");
        assert!(xml.contains(&format!("<node id=\"{}\"", id_b)), "missing node-b element");
        // Node type data
        assert!(xml.contains(">IP<"), "missing IP type data");
        assert!(xml.contains(">Domain<"), "missing Domain type data");

        // Edge element
        assert!(xml.contains("<edge id=\"e1\""), "missing edge element");
        assert!(xml.contains(&format!("source=\"{}\"", id_a)));
        assert!(xml.contains(&format!("target=\"{}\"", id_b)));
        assert!(xml.contains(">resolves-to<"), "missing edge label data");

        // Properties JSON present
        assert!(xml.contains("d_props"), "missing d_props data");
        assert!(xml.contains("d_edge_props"), "missing d_edge_props data");
    }

    #[test]
    fn graphml_escapes_special_chars_in_data() {
        let mut g = Graph::new();
        let mut n = Node::with_id("n1", "Info");
        // Value with XML-special characters that must be escaped
        n.properties.insert("label".into(), "A & B <test>".into());
        g.add_node(n);

        let xml = export_graphml_to_string(&g);

        // The raw characters must NOT appear unescaped in the XML.
        assert!(!xml.contains("A & B"), "& must be escaped to &amp;");
        assert!(!xml.contains("<test>"), "< must be escaped");
        // The escaped forms must be present.
        assert!(xml.contains("A &amp; B"), "expected &amp;");
        assert!(xml.contains("&lt;test&gt;"), "expected &lt; / &gt;");
    }

    #[test]
    fn graphml_empty_graph_is_valid_xml() {
        // An empty graph should produce well-formed XML with just the structure.
        let g = Graph::new();
        let xml = export_graphml_to_string(&g);
        assert!(xml.contains("<graphml"), "missing root element");
        assert!(xml.contains("</graphml>"), "missing closing root element");
        assert!(xml.contains("<graph id=\"G\""), "missing graph element");
        assert!(xml.contains("</graph>"), "missing closing graph element");
        // No node or edge elements
        assert!(!xml.contains("<node"), "empty graph must have no nodes");
        assert!(!xml.contains("<edge"), "empty graph must have no edges");
    }

    #[test]
    fn graphml_label_fallback_to_value_then_id() {
        let mut g = Graph::new();
        // Node with "value" property but no "label"
        let mut n1 = Node::with_id("n1", "IP");
        n1.properties.insert("value".into(), "1.2.3.4".into());
        g.add_node(n1);
        // Node with neither label nor value — should fall back to ID
        g.add_node(Node::with_id("n2", "Unknown"));

        let xml = export_graphml_to_string(&g);
        assert!(xml.contains(">1.2.3.4<"), "value should be used as label fallback");
        assert!(xml.contains(">n2<"),      "id should be used as label fallback");
    }

    // ── Index integrity (O(1) HashMap index + swap_remove) ─────────────────────
    //
    // These tests target the internal HashMap index introduced in the
    // optimisation pass.  When swap_remove reorders the Vec, every remaining
    // node/edge must still be reachable via find_node / find_node_mut.

    #[test]
    fn graph_new_is_empty() {
        let g = Graph::new();
        assert_eq!(g.nodes.len(), 0);
        assert_eq!(g.edges.len(), 0);
    }

    #[test]
    fn index_stable_after_removing_first_of_three() {
        // ┌──────────────────────────────────────────────────────────────────────┐
        // │ Layout before: [a, b, c]  →  remove a (index 0)                     │
        // │ swap_remove moves c to position 0: [c, b]                           │
        // │ Both c and b must still be findable.                                 │
        // └──────────────────────────────────────────────────────────────────────┘
        let mut g = Graph::new();
        let (a, b, c) = ("a", "b", "c");
        g.add_node(Node::with_id(a, "IP"));
        g.add_node(Node::with_id(b, "IP"));
        g.add_node(Node::with_id(c, "IP"));

        g.remove_node(a).expect("node-a should exist");

        assert!(g.find_node(a).is_none(), "removed node must be gone");
        assert!(g.find_node(b).is_some(), "node-b must still be findable");
        assert!(g.find_node(c).is_some(), "node-c must still be findable");
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn index_stable_after_removing_middle_of_three() {
        let mut g = Graph::new();
        g.add_node(Node::with_id("a", "IP"));
        g.add_node(Node::with_id("b", "IP"));
        g.add_node(Node::with_id("c", "IP"));

        g.remove_node("b").expect("node-b should exist");

        assert!(g.find_node("a").is_some());
        assert!(g.find_node("b").is_none());
        assert!(g.find_node("c").is_some());
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn index_stable_after_removing_last_of_three() {
        // Removing the last element needs no swap — just a truncation.
        let mut g = Graph::new();
        g.add_node(Node::with_id("a", "IP"));
        g.add_node(Node::with_id("b", "IP"));
        g.add_node(Node::with_id("c", "IP"));

        g.remove_node("c").expect("node-c should exist");

        assert!(g.find_node("a").is_some());
        assert!(g.find_node("b").is_some());
        assert!(g.find_node("c").is_none());
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn edge_index_stable_after_first_edge_removed() {
        // ┌──────────────────────────────────────────────────────────────────────┐
        // │ Two edges: [e1, e2].  Remove e1 (index 0).                          │
        // │ swap_remove brings e2 to position 0.  e2 must still be findable.    │
        // └──────────────────────────────────────────────────────────────────────┘
        let (mut g, id_a, id_b) = make_graph();
        // Add a third node so we can make two distinct edges.
        g.add_node(Node::with_id("c", "URL"));
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "first"));
        g.add_edge(Edge::with_id("e2", &id_a, "c",   "second"));

        g.remove_edge("e1").expect("e1 should exist");

        assert!(g.find_node(&id_a).is_some(), "node-a must survive");
        assert!(g.remove_edge("e2").is_some(), "e2 must still be removable");
        assert_eq!(g.edges.len(), 0);
    }

    #[test]
    fn remove_node_cascades_incident_edges_in_both_directions() {
        // A→B and C→A both reference A.  Removing A must delete both edges.
        let mut g = Graph::new();
        g.add_node(Node::with_id("A", "IP"));
        g.add_node(Node::with_id("B", "Domain"));
        g.add_node(Node::with_id("C", "Domain"));

        g.add_edge(Edge::with_id("e1", "A", "B", "fwd"));
        g.add_edge(Edge::with_id("e2", "C", "A", "rev"));
        assert_eq!(g.edges.len(), 2);

        g.remove_node("A");

        assert_eq!(g.edges.len(), 0, "both incident edges must be removed");
        assert!(g.find_node("B").is_some(), "B must survive");
        assert!(g.find_node("C").is_some(), "C must survive");
    }

    #[test]
    fn remove_node_only_removes_incident_edges() {
        // C→B is not incident to A.  Removing A must not touch it.
        let mut g = Graph::new();
        g.add_node(Node::with_id("A", "IP"));
        g.add_node(Node::with_id("B", "IP"));
        g.add_node(Node::with_id("C", "IP"));

        g.add_edge(Edge::with_id("e_ab", "A", "B", "link"));
        g.add_edge(Edge::with_id("e_cb", "C", "B", "link"));
        assert_eq!(g.edges.len(), 2);

        g.remove_node("A");

        assert_eq!(g.edges.len(), 1, "only A-incident edge must be removed");
        assert!(g.remove_edge("e_cb").is_some(), "C→B must survive");
    }

    #[test]
    fn self_loop_edge_accepted_and_cascades_with_node() {
        // An edge where source == target is valid (it marks a self-referential
        // relationship).  Removing the node must also remove the self-loop.
        let mut g = Graph::new();
        g.add_node(Node::with_id("A", "IP"));

        assert!(g.add_edge(Edge::with_id("loop", "A", "A", "self")),
            "self-loop should be accepted");
        assert_eq!(g.edges.len(), 1);

        g.remove_node("A");

        assert_eq!(g.edges.len(), 0, "self-loop must be removed with its node");
    }

    #[test]
    fn find_node_mut_persists_mutations() {
        // Mutations made through a mutable reference must be reflected in
        // subsequent immutable look-ups.
        let (mut g, id_a, _) = make_graph();

        {
            let node = g.find_node_mut(&id_a).expect("node-a must exist");
            node.properties.insert("value".into(), "mutated".into());
            node.node_type = "Domain".into();
        }

        let node = g.find_node(&id_a).expect("node-a must still be findable");
        assert_eq!(node.node_type, "Domain",       "type mutation must persist");
        assert_eq!(node.properties["value"], "mutated", "property mutation must persist");
    }

    #[test]
    fn deserialized_graph_find_node_works() {
        // After JSON round-trip the deserialized graph must have a working index
        // so find_node returns the correct nodes.
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "link"));

        let json = serde_json::to_string(&g).unwrap();
        let mut g2: Graph = serde_json::from_str(&json).unwrap();
        g2.rebuild_index();

        assert_eq!(g2.find_node(&id_a).map(|n| &n.node_type), Some(&"IP".to_string()));
        assert_eq!(g2.find_node(&id_b).map(|n| &n.node_type), Some(&"Domain".to_string()));
        assert!(g2.find_node("ghost").is_none());
    }

    #[test]
    fn deserialized_graph_allows_add_edge() {
        // After JSON round-trip, node_index must be populated so add_edge can
        // validate source/target existence (O(1) path).
        let (g, id_a, id_b) = make_graph();

        let json = serde_json::to_string(&g).unwrap();
        let mut g2: Graph = serde_json::from_str(&json).unwrap();
        g2.rebuild_index();

        // Without a working index add_edge would not find source/target and
        // would silently reject the edge.
        let ok = g2.add_edge(Edge::with_id("new-edge", &id_a, &id_b, "after-load"));
        assert!(ok, "add_edge after deserialization must succeed");
        assert_eq!(g2.edges.len(), 1);
    }

    #[test]
    fn load_graph_index_allows_add_edge() {
        // load_graph calls rebuild_index internally; verify end-to-end that a
        // freshly loaded file can accept new edges.
        let (g, id_a, id_b) = make_graph();

        let path = std::env::temp_dir()
            .join(format!("link_mage_idx_test_{}.json", Uuid::new_v4()));
        g.save_graph(&path).unwrap();

        let mut loaded = Graph::load_graph(&path).unwrap();
        std::fs::remove_file(&path).ok();

        let ok = loaded.add_edge(Edge::with_id("post-load", &id_a, &id_b, "linked"));
        assert!(ok, "add_edge on a loaded graph must succeed (index must be rebuilt)");
        assert_eq!(loaded.edges.len(), 1);
    }

    #[test]
    fn remove_then_readd_same_id() {
        // A node with a previously-used ID must be re-insertable after removal.
        let mut g = Graph::new();
        g.add_node(Node::with_id("recycled", "IP"));

        let removed = g.remove_node("recycled").expect("first removal must succeed");
        assert_eq!(removed.id, "recycled");
        assert!(g.find_node("recycled").is_none());

        // Re-insert with the same ID.
        assert!(g.add_node(Node::with_id("recycled", "Domain")),
            "re-adding a removed id must succeed");
        assert_eq!(g.find_node("recycled").unwrap().node_type, "Domain");
    }

    #[test]
    fn duplicate_edge_readd_after_remove() {
        // An edge id that was removed must be reusable.
        let (mut g, id_a, id_b) = make_graph();
        g.add_edge(Edge::with_id("e1", &id_a, &id_b, "first"));

        g.remove_edge("e1").unwrap();
        assert_eq!(g.edges.len(), 0);

        let ok = g.add_edge(Edge::with_id("e1", &id_a, &id_b, "second"));
        assert!(ok, "edge with a previously-removed id must be accepted");
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].label, "second");
    }

    #[test]
    fn edge_with_properties_preserved() {
        let (mut g, id_a, id_b) = make_graph();
        let mut e = Edge::with_id("e1", &id_a, &id_b, "link");
        e.properties.insert("weight".into(), "0.9".into());
        e.properties.insert("color".into(),  "red".into());

        g.add_edge(e);

        // Properties survive through the index (not stored there, but the Vec
        // must be unaffected by index operations).
        let edge = g.edges.iter().find(|e| e.id == "e1").unwrap();
        assert_eq!(edge.properties["weight"], "0.9");
        assert_eq!(edge.properties["color"],  "red");
    }

    #[test]
    fn stress_many_nodes_all_findable() {
        // Add 500 nodes with generated IDs and verify every one is reachable
        // via the O(1) index.  This would time-out with a linear scan on larger
        // graphs, making it a reliable performance regression canary.
        const N: usize = 500;
        let mut g = Graph::new();
        let mut ids: Vec<String> = Vec::with_capacity(N);

        for i in 0..N {
            let id = format!("node-{i:04}");
            g.add_node(Node::with_id(&id, "IP"));
            ids.push(id);
        }

        assert_eq!(g.nodes.len(), N);
        for id in &ids {
            assert!(
                g.find_node(id).is_some(),
                "node '{id}' must be findable after bulk insert",
            );
        }
    }

    #[test]
    fn stress_interleaved_add_remove_consistent() {
        // Add 20 nodes, remove every other one, then add 10 more.
        // Every surviving node must be findable; removed ones must not be.
        let mut g = Graph::new();
        let ids: Vec<String> = (0..20).map(|i| format!("n{i}")).collect();

        for id in &ids {
            g.add_node(Node::with_id(id, "IP"));
        }

        let (removed, kept): (Vec<_>, Vec<_>) = ids.iter().enumerate()
            .partition(|(i, _)| i % 2 == 0);

        for (_, id) in &removed {
            g.remove_node(id).expect("should exist before removal");
        }

        assert_eq!(g.nodes.len(), kept.len());

        for (_, id) in &kept {
            assert!(g.find_node(id).is_some(), "kept node '{id}' must be findable");
        }
        for (_, id) in &removed {
            assert!(g.find_node(id).is_none(), "removed node '{id}' must be gone");
        }

        // Add 10 fresh nodes — index must still be valid.
        for i in 20..30 {
            let id = format!("new-{i}");
            assert!(g.add_node(Node::with_id(&id, "Domain")));
            assert!(g.find_node(&id).is_some());
        }
    }

    #[test]
    fn stress_add_and_remove_edges_consistent() {
        // Build a star graph: 1 hub + 50 spokes.  Remove every other spoke node
        // and verify that the corresponding edges are cascade-removed, while the
        // remaining edges are still reachable.
        let mut g = Graph::new();
        g.add_node(Node::with_id("hub", "Domain"));

        let spoke_ids: Vec<String> = (0..50).map(|i| format!("spoke-{i}")).collect();
        let edge_ids:  Vec<String> = (0..50).map(|i| format!("edge-{i}")).collect();

        for (sid, eid) in spoke_ids.iter().zip(edge_ids.iter()) {
            g.add_node(Node::with_id(sid, "IP"));
            g.add_edge(Edge::with_id(eid, "hub", sid, "points-to"));
        }
        assert_eq!(g.nodes.len(), 51);
        assert_eq!(g.edges.len(), 50);

        // Remove odd-indexed spokes.
        let (removed_spokes, kept_spokes): (Vec<_>, Vec<_>) =
            spoke_ids.iter().enumerate().partition(|(i, _)| i % 2 == 1);

        for (_, sid) in &removed_spokes {
            g.remove_node(sid).expect("spoke must exist");
        }

        assert_eq!(g.nodes.len(), 1 + kept_spokes.len(),
            "hub + kept spokes must remain");
        assert_eq!(g.edges.len(), kept_spokes.len(),
            "only edges to kept spokes must survive");

        // Every kept edge must still be removable via the index.
        for (i, _) in &kept_spokes {
            let eid = format!("edge-{i}");
            assert!(
                g.remove_edge(&eid).is_some(),
                "kept edge '{eid}' must be removable",
            );
        }
    }
}
