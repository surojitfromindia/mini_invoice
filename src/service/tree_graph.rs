use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use crate::errors::app_error::AppError;

// Generic tree graph helper for service-layer models that start as flat rows.
//
// The important idea is:
// - each row has a unique id
// - each row may point to a parent id
// - we rebuild the parent/child structure in memory once
// - then we can render either a flat list or a nested tree from the same data
#[derive(Debug)]
pub struct TreeGraph<T, Id>
where
    Id: Eq + Hash + Copy + Debug,
{
    node_label: &'static str,
    nodes_by_id: HashMap<Id, T>,
    children_by_parent_id: HashMap<Id, Vec<Id>>,
    root_ids: Vec<Id>,
}

impl<T, Id> TreeGraph<T, Id>
where
    Id: Eq + Hash + Copy + Debug,
{
    pub fn try_new(
        nodes: Vec<T>,
        node_id: impl Fn(&T) -> Id,
        parent_id: impl Fn(&T) -> Option<Id>,
        node_label: &'static str,
    ) -> Result<Self, AppError> {
        let mut nodes_by_id = HashMap::with_capacity(nodes.len());

        // First, move every row into a lookup table so we can find any node by
        // id later without scanning the full list again.
        for node in nodes {
            let id = node_id(&node);

            if nodes_by_id.insert(id, node).is_some() {
                return Err(AppError::InternalServer(format!(
                    "Duplicate {node_label} `{:?}` encountered while building tree graph",
                    id,
                )));
            }
        }

        let mut children_by_parent_id: HashMap<Id, Vec<Id>> = HashMap::new();
        let mut root_ids = Vec::new();

        // Next, turn the flat rows into a parent -> children map.
        // Rows with no parent become roots.
        for (id, node) in &nodes_by_id {
            if let Some(parent_id) = parent_id(node) {
                if !nodes_by_id.contains_key(&parent_id) {
                    return Err(AppError::InternalServer(format!(
                        "Missing parent node `{:?}` while resolving {node_label} `{:?}`",
                        parent_id, id,
                    )));
                }

                children_by_parent_id
                    .entry(parent_id)
                    .or_default()
                    .push(*id);
            } else {
                root_ids.push(*id);
            }
        }

        Ok(Self {
            node_label,
            nodes_by_id,
            children_by_parent_id,
            root_ids,
        })
    }

    pub fn len(&self) -> usize {
        self.nodes_by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes_by_id.is_empty()
    }

    pub fn contains(&self, node_id: Id) -> bool {
        self.nodes_by_id.contains_key(&node_id)
    }

    pub fn get(&self, node_id: Id) -> Option<&T> {
        self.nodes_by_id.get(&node_id)
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.nodes_by_id.values()
    }

    pub fn nodes_by_id(&self) -> &HashMap<Id, T> {
        &self.nodes_by_id
    }

    pub fn flatten<R, MapFn, SortFn>(
        &self,
        map_node: &MapFn,
        compare_ids: &SortFn,
    ) -> Result<Vec<R>, AppError>
    where
        MapFn: Fn(&T) -> Result<R, AppError>,
        SortFn: Fn(Id, Id, &HashMap<Id, T>) -> Ordering,
    {
        let mut flat_items = Vec::with_capacity(self.nodes_by_id.len());
        let mut visited = HashSet::with_capacity(self.nodes_by_id.len());

        // We still walk the data like a tree so parents appear before their
        // children in the flat output.
        for root_id in self.sorted_root_ids(compare_ids) {
            self.flatten_from(
                root_id,
                &mut visited,
                &mut flat_items,
                &mut Vec::new(),
                map_node,
                compare_ids,
            )?;
        }

        self.ensure_all_nodes_visited(&visited)?;
        Ok(flat_items)
    }

    pub fn build_tree<R, MapFn, SortFn>(
        &self,
        map_node: &MapFn,
        compare_ids: &SortFn,
    ) -> Result<Vec<R>, AppError>
    where
        MapFn: Fn(&T, Vec<R>) -> Result<R, AppError>,
        SortFn: Fn(Id, Id, &HashMap<Id, T>) -> Ordering,
    {
        let mut visited = HashSet::with_capacity(self.nodes_by_id.len());
        let mut tree = Vec::with_capacity(self.root_ids.len());

        // Tree rendering uses the same traversal rules as flat rendering, but
        // nests each child list inside its parent node.
        for root_id in self.sorted_root_ids(compare_ids) {
            tree.push(self.build_tree_from(
                root_id,
                &mut visited,
                &mut Vec::new(),
                map_node,
                compare_ids,
            )?);
        }

        self.ensure_all_nodes_visited(&visited)?;
        Ok(tree)
    }

    fn flatten_from<R, MapFn, SortFn>(
        &self,
        node_id: Id,
        visited: &mut HashSet<Id>,
        flat_items: &mut Vec<R>,
        stack: &mut Vec<Id>,
        map_node: &MapFn,
        compare_ids: &SortFn,
    ) -> Result<(), AppError>
    where
        MapFn: Fn(&T) -> Result<R, AppError>,
        SortFn: Fn(Id, Id, &HashMap<Id, T>) -> Ordering,
    {
        // `stack` tracks the current path we are walking.
        // If we see the same id twice on that path, the data has a cycle.
        if stack.contains(&node_id) {
            return Err(AppError::InternalServer(format!(
                "Cycle detected in {node_label} tree at node `{:?}`",
                node_id,
                node_label = self.node_label
            )));
        }

        // `visited` makes sure we do not emit the same node twice.
        if !visited.insert(node_id) {
            return Err(AppError::InternalServer(format!(
                "Duplicate {node_label} `{:?}` encountered while flattening tree",
                node_id,
                node_label = self.node_label
            )));
        }

        stack.push(node_id);
        let node = self.node(node_id)?;
        flat_items.push(map_node(node)?);

        for child_id in self.sorted_child_ids(node_id, compare_ids)? {
            self.flatten_from(child_id, visited, flat_items, stack, map_node, compare_ids)?;
        }

        stack.pop();
        Ok(())
    }

    fn build_tree_from<R, MapFn, SortFn>(
        &self,
        node_id: Id,
        visited: &mut HashSet<Id>,
        stack: &mut Vec<Id>,
        map_node: &MapFn,
        compare_ids: &SortFn,
    ) -> Result<R, AppError>
    where
        MapFn: Fn(&T, Vec<R>) -> Result<R, AppError>,
        SortFn: Fn(Id, Id, &HashMap<Id, T>) -> Ordering,
    {
        // Same cycle check as the flat traversal.
        if stack.contains(&node_id) {
            return Err(AppError::InternalServer(format!(
                "Cycle detected in {node_label} tree at node `{:?}`",
                node_id,
                node_label = self.node_label
            )));
        }

        // A node should only appear once in the final tree.
        if !visited.insert(node_id) {
            return Err(AppError::InternalServer(format!(
                "Duplicate {node_label} `{:?}` encountered while building tree",
                node_id,
                node_label = self.node_label
            )));
        }

        stack.push(node_id);
        let node = self.node(node_id)?;
        let children = self
            .sorted_child_ids(node_id, compare_ids)?
            .into_iter()
            .map(|child_id| self.build_tree_from(child_id, visited, stack, map_node, compare_ids))
            .collect::<Result<Vec<_>, _>>()?;
        stack.pop();

        map_node(node, children)
    }

    fn ensure_all_nodes_visited(&self, visited: &HashSet<Id>) -> Result<(), AppError> {
        // If some nodes never got visited, the graph is broken:
        // either disconnected, cyclic, or both.
        if visited.len() == self.nodes_by_id.len() {
            return Ok(());
        }

        let missing_ids: Vec<String> = self
            .nodes_by_id
            .keys()
            .filter(|node_id| !visited.contains(node_id))
            .map(|node_id| format!("{node_id:?}"))
            .collect();

        Err(AppError::InternalServer(format!(
            "{node_label} tree is disconnected or cyclic; unreachable node ids: {}",
            missing_ids.join(", "),
            node_label = self.node_label
        )))
    }

    fn sorted_root_ids<SortFn>(&self, compare_ids: &SortFn) -> Vec<Id>
    where
        SortFn: Fn(Id, Id, &HashMap<Id, T>) -> Ordering,
    {
        let mut root_ids = self.root_ids.clone();
        root_ids.sort_by(|left, right| compare_ids(*left, *right, &self.nodes_by_id));
        root_ids
    }

    fn sorted_child_ids<SortFn>(
        &self,
        node_id: Id,
        compare_ids: &SortFn,
    ) -> Result<Vec<Id>, AppError>
    where
        SortFn: Fn(Id, Id, &HashMap<Id, T>) -> Ordering,
    {
        let mut child_ids = self
            .children_by_parent_id
            .get(&node_id)
            .cloned()
            .unwrap_or_default();
        child_ids.sort_by(|left, right| compare_ids(*left, *right, &self.nodes_by_id));
        Ok(child_ids)
    }

    fn node(&self, node_id: Id) -> Result<&T, AppError> {
        self.nodes_by_id.get(&node_id).ok_or_else(|| {
            AppError::InternalServer(format!(
                "{node_label} `{:?}` is missing from the current tree graph",
                node_id,
                node_label = self.node_label
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestNode {
        id: i32,
        parent_id: Option<i32>,
    }

    #[derive(Debug, PartialEq)]
    struct TestTreeNode {
        id: i32,
        children: Vec<TestTreeNode>,
    }

    #[test]
    fn tree_graph_flattens_and_builds_tree_in_sorted_order() {
        let graph = TreeGraph::try_new(
            vec![
                TestNode {
                    id: 2,
                    parent_id: Some(1),
                },
                TestNode {
                    id: 1,
                    parent_id: None,
                },
                TestNode {
                    id: 3,
                    parent_id: Some(1),
                },
            ],
            |node| node.id,
            |node| node.parent_id,
            "test node",
        )
        .unwrap();

        let flat = graph
            .flatten(&|node| Ok(node.id), &|left, right, _| left.cmp(&right))
            .unwrap();

        assert_eq!(flat, vec![1, 2, 3]);

        let tree = graph
            .build_tree(
                &|node, children| {
                    Ok(TestTreeNode {
                        id: node.id,
                        children,
                    })
                },
                &|left, right, _| left.cmp(&right),
            )
            .unwrap();

        assert_eq!(
            tree,
            vec![TestTreeNode {
                id: 1,
                children: vec![
                    TestTreeNode {
                        id: 2,
                        children: vec![]
                    },
                    TestTreeNode {
                        id: 3,
                        children: vec![]
                    },
                ],
            }]
        );
    }

    #[test]
    fn tree_graph_rejects_missing_parent() {
        let err = TreeGraph::try_new(
            vec![TestNode {
                id: 2,
                parent_id: Some(1),
            }],
            |node| node.id,
            |node| node.parent_id,
            "test node",
        )
        .unwrap_err();

        match err {
            AppError::InternalServer(message) => {
                assert!(message.contains("Missing parent node"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
