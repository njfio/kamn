pub(crate) struct ActorInput<'a> {
    pub(crate) role: &'a str,
    pub(crate) pid: u64,
    pub(crate) did: &'a str,
    pub(crate) escrow: &'a str,
    pub(crate) projection: String,
    pub(crate) private: Option<String>,
    pub(crate) handoff_authorized: bool,
    pub(crate) handoff_as_string: bool,
    pub(crate) include_release: bool,
    pub(crate) duplicate_fund: bool,
    pub(crate) release_error: bool,
    pub(crate) receipt_digest_mismatch: bool,
    pub(crate) public_fact_drift: bool,
}

impl<'a> ActorInput<'a> {
    pub(crate) fn new(
        role: &'a str,
        pid: u64,
        did: &'a str,
        escrow: &'a str,
        projection: char,
    ) -> Self {
        Self {
            role,
            pid,
            did,
            escrow,
            projection: sha(projection),
            private: None,
            handoff_authorized: false,
            handoff_as_string: false,
            include_release: true,
            duplicate_fund: false,
            release_error: false,
            receipt_digest_mismatch: false,
            public_fact_drift: false,
        }
    }

    pub(crate) fn with_private(mut self, value: String) -> Self {
        self.private = Some(value);
        self
    }

    pub(crate) fn with_handoff_authorized(mut self, value: bool) -> Self {
        self.handoff_authorized = value;
        self
    }

    pub(crate) fn with_handoff_as_string(mut self, value: bool) -> Self {
        self.handoff_as_string = value;
        self
    }

    pub(crate) fn with_release(mut self, value: bool) -> Self {
        self.include_release = value;
        self
    }

    pub(crate) fn with_duplicate_fund(mut self, value: bool) -> Self {
        self.duplicate_fund = value;
        self
    }

    pub(crate) fn with_release_error(mut self, value: bool) -> Self {
        self.release_error = value;
        self
    }

    pub(crate) fn with_receipt_digest_mismatch(mut self, value: bool) -> Self {
        self.receipt_digest_mismatch = value;
        self
    }

    pub(crate) fn with_public_fact_drift(mut self, value: bool) -> Self {
        self.public_fact_drift = value;
        self
    }
}

pub(crate) fn runtime_receipts(input: &ActorInput<'_>) -> String {
    tools(input)
        .iter()
        .enumerate()
        .map(|(index, tool)| receipt_json(input, index, tool))
        .collect::<Vec<_>>()
        .join(",")
}

fn tools(input: &ActorInput<'_>) -> [&'static str; 5] {
    match input.role {
        "agent_a" => [
            "register",
            "create_task",
            "fund_escrow",
            if input.duplicate_fund {
                "fund_escrow"
            } else if input.include_release {
                "release_escrow"
            } else {
                "query_task"
            },
            "query_participant_task_projection",
        ],
        "agent_b" => [
            "register",
            "accept_task",
            "complete_task",
            "query_task",
            "query_participant_task_projection",
        ],
        _ => [
            "register",
            "query_task",
            "query_task",
            "query_task",
            "query_verifier_task_projection",
        ],
    }
}

fn receipt_json(input: &ActorInput<'_>, index: usize, tool: &str) -> String {
    let is_error = input.release_error && tool == "release_escrow";
    let outcome = if is_error { "error" } else { "success" };
    let mut public_result = if is_error {
        "{}".to_owned()
    } else {
        public_result_json(input.role, tool)
    };
    if input.public_fact_drift && tool == "fund_escrow" {
        public_result = public_result.replace("escrow-live-7099", "escrow-other");
    }
    let digest = if input.receipt_digest_mismatch && tool == "release_escrow" {
        sha('f')
    } else {
        response_digests(input)[index].clone()
    };
    format!(
        r#"{{"request_id":{},"tool":"{}","outcome":"{outcome}","digest":"{}","public_result":{public_result}}}"#,
        index + 1,
        tool,
        digest,
    )
}

fn public_result_json(role: &str, tool: &str) -> String {
    match tool {
        "register" => format!(r#"{{"did":"kamn:did:{}"}}"#, role_suffix(role)),
        "create_task" => task_result("submitted"),
        "accept_task" => task_result("accepted"),
        "complete_task" => task_result("completed"),
        "fund_escrow" => r#"{"escrow_id":"escrow-live-7099","state":"funded"}"#.to_owned(),
        "release_escrow" => r#"{"escrow_id":"escrow-live-7099","state":"released"}"#.to_owned(),
        "query_participant_task_projection" => projection_result(role, "participant-private"),
        "query_verifier_task_projection" => projection_result(role, "restricted-public"),
        _ => r#"{"task_id":"task-live-7099","state":"accepted"}"#.to_owned(),
    }
}

fn task_result(state: &str) -> String {
    format!(r#"{{"task_id":"task-live-7099","state":"{state}"}}"#)
}

fn projection_result(role: &str, scope: &str) -> String {
    let participant = match role {
        "agent_a" => r#", "participant_role":"creator""#,
        "agent_b" => r#", "participant_role":"provider""#,
        _ => "",
    };
    format!(
        r#"{{"task_id":"task-live-7099","escrow_id":"escrow-live-7099","settlement_tx_signature":"devnet-signature-7099","settlement_commitment":"finalized","public_commitment":"{}","view_scope":"{scope}"{participant}}}"#,
        sha('d')
    )
}

fn role_suffix(role: &str) -> char {
    match role {
        "agent_a" => 'a',
        "agent_b" => 'b',
        _ => 'c',
    }
}

pub(crate) fn response_digests(input: &ActorInput<'_>) -> [String; 5] {
    let projection = if input.projection == sha('f') {
        sha('3')
    } else {
        input.projection.clone()
    };
    [sha('a'), sha('b'), sha('c'), sha('d'), projection]
}

pub(crate) fn sha(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}
