use gpui::{
    AnyElement, App, DefiniteLength, Entity, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _,Context, Render, StyleRefinement, Styled, Window, div, px,
};
use gpui_component::{IconName, Size, h_flex, v_flex};
use std::collections::HashMap;

pub struct Tree {
    state: Entity<TreeState>,
    _style: StyleRefinement,
    _size: Size,
    _height: Option<DefiniteLength>,
    _appearance: bool,
    _bordered: bool,
    _focus_bordered: bool,
}

pub struct TreeState {
    pub nodes: Vec<TreeNode>,
    pub node_states: HashMap<usize, NodeState>,
    pub selected_node_id: Option<usize>,
}

impl EventEmitter<TreeEvent> for TreeState {}

#[derive(Clone)]
pub struct TreeNode {
    pub id: usize,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub is_dir: bool,
}

pub struct NodeState {
    pub expanded: bool,
}

pub enum TreeEvent {
    OnClick(usize),
    OnDoubleClick(usize),
    OnRightClick(usize),
}

impl TreeState {
    pub fn toggle_expanded(&mut self, node_id: usize) {
        if let Some(node_state) = self.node_states.get_mut(&node_id) {
            node_state.expanded = !node_state.expanded;
        } else {
            self.node_states
                .insert(node_id, NodeState { expanded: true });
        }
    }

    pub fn select_node(&mut self, node_id: usize) {
        self.selected_node_id = Some(node_id);
    }
}

impl Tree {
    pub fn new(state: &Entity<TreeState>) -> Self {
        Self {
            state: state.clone(),
            _size: Size::default(),
            _style: StyleRefinement::default(),
            _height: None,
            _appearance: true,
            _bordered: true,
            _focus_bordered: true,
        }
    }
}

impl Render for Tree {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nodes = self.state.read(cx).nodes.clone();
        v_flex()
            .id(("tree", self.state.entity_id()))
            .size_full()
            .children(
                nodes
                    .into_iter()
                    .map(|node| self.render_node(&node, 0, window, cx)),
            )
    }
}

impl Tree {
    fn render_node(
        &self,
        node: &TreeNode,
        depth: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let state = self.state.read(cx);
        let node_state = state
            .node_states
            .get(&node.id)
            .unwrap_or(&NodeState { expanded: false });
        let is_selected = state.selected_node_id == Some(node.id);

        v_flex()
            .pl(px(depth as f32 * 5.0))
            .child(
                h_flex()
                    .items_center()
                    .gap_1()
                    .bg(if is_selected {
                        gpui::red()
                    } else {
                        gpui::transparent_black()
                    })
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let state = self.state.clone();
                        let node_id = node.id;
                        move |event, _, cx| {
                            if event.click_count == 2 {
                                state.update(cx, |_, cx| {
                                    cx.emit(TreeEvent::OnDoubleClick(node_id));
                                });
                            } else {
                                state.update(cx, |state, cx| {
                                    state.toggle_expanded(node_id);
                                    state.select_node(node_id);
                                    cx.emit(TreeEvent::OnClick(node_id));
                                });
                            }
                        }
                    })
                    .on_mouse_down(gpui::MouseButton::Right, {
                        let state = self.state.clone();
                        let node_id = node.id;
                        move |_, _, cx| {
                            state.update(cx, |_, cx| {
                                cx.emit(TreeEvent::OnRightClick(node_id));
                            });
                        }
                    })
                    .child(div().id(node.id).child(if node.is_dir {
                        if node_state.expanded {
                            IconName::FolderOpen
                        } else {
                            IconName::Folder
                        }
                    } else {
                        IconName::Star
                    }))
                    .child(div().child(node.label.clone())),
            )
            .children(if node_state.expanded {
                node.children
                    .iter()
                    .map(|child| self.render_node(child, depth + 1, window, cx))
                    .collect::<Vec<_>>()
            } else {
                vec![]
            })
            .into_any_element()
    }
}
