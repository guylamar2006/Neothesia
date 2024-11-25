use smallvec::SmallVec;

use crate::{Element, LayoutCtx, Node, ParentLayout, Tree, Widget};

pub struct Row<MSG> {
    children: SmallVec<[Element<MSG>; 4]>,
    gap: f32,
}

impl<MSG> Default for Row<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> Row<MSG> {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            gap: 0.0,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn push(mut self, widget: impl Into<Element<MSG>>) -> Self {
        self.children.push(widget.into());
        self
    }

    pub fn when(self, v: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if v {
            f(self)
        } else {
            self
        }
    }
}

impl<MSG> Widget<MSG> for Row<MSG> {
    type State = ();

    fn children(&self) -> &[Element<MSG>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Element<MSG>] {
        &mut self.children
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        let mut children = Vec::with_capacity(self.children.len());

        let mut item_layout = ParentLayout {
            x: parent.x,
            y: parent.y,
            w: parent.w,
            h: parent.h,
        };

        let mut total_width = 0.0;

        for (ch, tree) in self.children.iter().zip(tree.children.iter_mut()) {
            let node = ch.as_widget().layout(tree, &item_layout, ctx);

            item_layout.x += node.w;
            item_layout.w -= node.w;

            item_layout.x += self.gap;
            item_layout.w -= self.gap;

            total_width += node.w;
            total_width += self.gap;

            children.push(node);
        }

        total_width -= self.gap;
        total_width = total_width.max(0.0);

        Node {
            x: parent.x,
            y: parent.y,
            w: total_width,
            h: parent.h,
            children,
        }
    }
}

impl<MSG: 'static> From<Row<MSG>> for Element<MSG> {
    fn from(value: Row<MSG>) -> Self {
        Element::new(value)
    }
}
