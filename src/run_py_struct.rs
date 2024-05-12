pub mod run_py_struct {
    use serde::{Deserialize, Serialize};
    use chrono::prelude::*;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct RunPy {
        pub id: usize,
        pub description: String,
        pub py_script: String,
        pub created_at: DateTime<Utc>,
    }
    
}