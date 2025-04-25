use super::colors::{Color, ColorScheme};

/// Information about a group of symbols in an alphabet
#[derive(Debug, Clone)]
pub struct GroupInfo {
    /// Name of the group
    pub name: String,
    /// Description of the group
    pub description: String,
    /// Color group identifier
    /// Color group identifier
    pub color_group: String,
    /// Foreground color override
    pub foreground_color: Option<Color>,
    /// Background color override
    pub background_color: Option<Color>,
    /// Starting symbol index (inclusive)
    pub start: usize,
    /// Ending symbol index (exclusive)
    pub end: usize,
    /// Whether the group is mutable
    pub mutable: bool,
    /// Whether the group is visible
    pub visible: bool,
    /// Parent group (if any)
    pub parent: Option<Box<GroupInfo>>,
    /// Child groups
    pub children: Vec<GroupInfo>,
}

impl GroupInfo {
    /// Create a new group information structure
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            color_group: String::new(),
            foreground_color: None,
            background_color: None,
            start: 0,
            end: 0,
            mutable: true,
            visible: true,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Check if a symbol is in this group
    pub fn contains_symbol(&self, symbol: usize) -> bool {
        symbol >= self.start && symbol < self.end
    }

    /// Add a child group
    pub fn add_child(&mut self, mut child: GroupInfo) {
        child.parent = Some(Box::new(self.clone()));
        self.children.push(child);
    }

    /// Get the root group (topmost parent)
    pub fn root(&self) -> &GroupInfo {
        let mut current = self;
        while let Some(parent) = &current.parent {
            current = parent;
        }
        current
    }

    /// Get all ancestors of this group
    pub fn ancestors(&self) -> Vec<&GroupInfo> {
        let mut ancestors = Vec::new();
        let mut current = self;
        while let Some(parent) = &current.parent {
            ancestors.push(parent.as_ref());
            current = parent;
        }
        ancestors
    }

    /// Get all descendants of this group
    pub fn descendants(&self) -> Vec<&GroupInfo> {
        let mut descendants = Vec::new();
        for child in &self.children {
            descendants.push(child);
            descendants.extend(child.descendants());
        }
        descendants
    }

    /// Find a group by name (searches this group and all descendants)
    pub fn find_by_name(&self, name: &str) -> Option<&GroupInfo> {
        if self.name == name {
            Some(self)
        } else {
            for child in &self.children {
                if let Some(found) = child.find_by_name(name) {
                    return Some(found);
                }
            }
            None
        }
    }

    /// Get the colors for this group from a color scheme
    pub fn get_colors(&self, scheme: &ColorScheme) -> Option<(Color, Color)> {
        // If colors are explicitly set, use those
        if let (Some(fg), Some(bg)) = (self.foreground_color, self.background_color) {
            return Some((fg, bg));
        }

        // Try to get colors from the scheme based on color group
        if !self.color_group.is_empty() {
            // Use a simple hash of the color group name to pick a color pair
            let hash: usize = self.color_group.chars().fold(0, |acc, c| acc.wrapping_add(c as usize));
            let index = hash % scheme.background_colors.len();
            scheme.get_color_pair(index)
        } else {
            // Default to the first color pair in the scheme
            scheme.get_color_pair(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_hierarchy() {
        let mut root = GroupInfo::new("root".to_string());
        root.start = 1;
        root.end = 10;

        let mut child1 = GroupInfo::new("child1".to_string());
        child1.start = 1;
        child1.end = 5;

        let mut child2 = GroupInfo::new("child2".to_string());
        child2.start = 5;
        child2.end = 10;

        let grandchild = GroupInfo::new("grandchild".to_string());

        child1.add_child(grandchild);
        root.add_child(child1);
        root.add_child(child2);

        assert!(root.contains_symbol(5));
        assert!(!root.contains_symbol(10));

        let found = root.find_by_name("grandchild");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "grandchild");

        let descendants = root.descendants();
        assert_eq!(descendants.len(), 3);
    }
}
