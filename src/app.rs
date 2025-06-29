pub struct AppState {
    pub selected_group: usize,
    pub current_selection: usize,
    pub focus: usize, // 0 = groups, 1 = connections
    pub search_mode: bool,
    pub search_query: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_group: 0,
            current_selection: 0,
            focus: 0,
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn reset_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.focus = 0;
        self.current_selection = 0;
    }

    pub fn enter_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.focus = 1;
        self.current_selection = 0;
    }

    pub fn move_down(&mut self, group_count: usize, conn_count: usize) {
        if self.focus == 0 {
            self.selected_group = (self.selected_group + 1) % group_count;
        } else {
            self.current_selection = (self.current_selection + 1) % conn_count.max(1);
        }
    }

    pub fn move_up(&mut self, group_count: usize, conn_count: usize) {
        if self.focus == 0 {
            self.selected_group = if self.selected_group > 0 {
                self.selected_group - 1
            } else {
                group_count.saturating_sub(1)
            };
        } else {
            self.current_selection = if self.current_selection > 0 {
                self.current_selection - 1
            } else {
                conn_count.saturating_sub(1)
            };
        }
    }

    pub fn handle_left(&mut self) {
        if !self.search_mode {
            self.focus = 0;
        }
    }

    pub fn handle_right(&mut self) {
        if !self.search_mode {
            self.focus = 1;
        }
    }
} 