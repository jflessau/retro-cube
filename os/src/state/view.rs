pub enum View {
    Clock,
    Weather,
    Mailbox,
}

impl View {
    pub fn next(&self) -> Self {
        match self {
            View::Clock => View::Weather,
            View::Weather => View::Mailbox,
            View::Mailbox => View::Clock,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            View::Mailbox => View::Weather,
            View::Weather => View::Clock,
            View::Clock => View::Mailbox,
        }
    }
}
