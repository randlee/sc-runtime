#[cfg(test)]
mod layout_tests {
    use schema::{RowRecord, TableName};

    use crate::Rollup;

    #[test]
    fn membership_matches_the_layout_engine() {
        let flags = crate::layout_flags().expect("membership");
        assert_eq!(flags.sqlite_only, vec!["local_notes".to_owned()]);
        assert_eq!(flags.remote_only, vec!["remote_audit".to_owned()]);
        assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
        assert!(
            crate::REMOTE_ENGINE == "postgres" || crate::REMOTE_ENGINE == "mysql",
            "{}",
            crate::REMOTE_ENGINE
        );
        let rows = [RowRecord {
            table: TableName::widgets(),
            payload: br#"{"id":"w1"}"#.to_vec(),
        }];
        assert_eq!(
            crate::CopyRollup.rollup(&rows, &flags.column_on_one_side),
            rows
        );
    }
}
