use crate::GameId;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct ProcessingData {
    unprocessed_split_moves: Vec<Vec<GameId>>,
    impossible_nimbers: Vec<bool>,
    smallest_possible_nimber: usize,
}

impl ProcessingData {
    pub fn new(moves: Vec<Vec<GameId>>) -> ProcessingData {
        ProcessingData {
            unprocessed_split_moves: moves,
            impossible_nimbers: Vec::new(),
            smallest_possible_nimber: 0,
        }
    }
    pub fn get_smallest_possible_nimber(&self) -> usize {
        self.smallest_possible_nimber
    }
    pub fn mark_impossible(&mut self, nimber: usize) {
        if nimber >= self.impossible_nimbers.len() {
            self.impossible_nimbers.resize(nimber + 1, false);
        }
        self.impossible_nimbers[nimber] = true;

        while self
            .impossible_nimbers
            .get(self.smallest_possible_nimber)
            .copied()
            .unwrap_or(false)
        {
            self.smallest_possible_nimber += 1;
        }
    }
    pub fn pop_unprocessed_move(&mut self) -> Option<Vec<GameId>> {
        self.unprocessed_split_moves.pop()
    }
    pub fn append_unprocessed_moves(&mut self, other: Vec<Vec<GameId>>) {
        self.unprocessed_split_moves.extend(other);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum EntryData {
    Stub {},
    Processing { data: ProcessingData },
    Done { nimber: usize },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Entry {
    pub data: EntryData,
    pub max_nimber: Option<usize>,
}

impl Entry {
    pub fn new(max_nimber: Option<usize>) -> Entry {
        match max_nimber {
            Some(0) => Self {
                max_nimber: Some(0),
                data: EntryData::Done { nimber: 0 },
            },
            _ => Self {
                max_nimber,
                data: EntryData::Stub {},
            },
        }
    }
    pub fn is_stub(&self) -> bool {
        matches!(self.data, EntryData::Stub {})
    }
    pub fn get_nimber(&self) -> Option<usize> {
        match &self.data {
            EntryData::Done { nimber } => Some(*nimber),
            _ => None,
        }
    }

    pub fn get_smallest_possible_nimber(&self) -> Option<usize> {
        match &self.data {
            EntryData::Processing { data } => Some(data.get_smallest_possible_nimber()),
            _ => None,
        }
    }
    pub fn mark_impossible(&mut self, nimber: usize) {
        if let EntryData::Processing { data } = &mut self.data {
            data.mark_impossible(nimber);
        }
    }
    pub fn pop_unprocessed_move(&mut self) -> Option<Option<Vec<GameId>>> {
        if let EntryData::Processing { data } = &mut self.data {
            Some(data.pop_unprocessed_move())
        } else {
            None
        }
    }
    pub fn append_unprocessed_moves(&mut self, other: Vec<Vec<GameId>>) {
        if let EntryData::Processing { data } = &mut self.data {
            data.append_unprocessed_moves(other);
        }
    }
}
