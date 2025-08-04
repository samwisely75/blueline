//! Test file to verify the fix for issue #58
//! This test verifies that single-line JSON responses are not broken into multiple lines

#[cfg(test)]
mod tests {
    use crate::repl::view_models::ViewModel;

    #[test]
    fn test_single_line_json_response_not_wrapped() {
        // Create a ViewModel
        let mut vm = ViewModel::new();
        vm.update_terminal_size(80, 24);

        // The JSON response from the issue
        let json_response = r#"{"took":2,"timed_out":false,"_shards":{"total":1,"successful":1,"skipped":0,"failed":0},"hits":{"total":{"value":4,"relation":"eq"},"max_score":1.0,"hits":[{"_index":"nihongo_sensei","_id":"nOp6YpgBVKdRCT3-JtpC","_score":1.0,"_source":{"@timestamp": "2025-07-31T21:53:11Z","english": "hello","japanese": "こんにちは"}},{"_index":"nihongo_sensei","_id":"QOp7YpgBVKdRCT3-Lu54","_score":1.0,"_source":{"@timestamp": "2025-07-31T21:53:11Z","english": "hello my name a Borat","japanese": "こんにちは、私名前 Borat です"}},{"_index":"nihongo_sensei","_id":"J5WMYpgBahU36fBrMGy5","_score":1.0,"_ignored":["english.keyword"],"_source":{"@timestamp": "2025-07-31T21:53:11Z","english": "My name-a Borat. I come from Kazakhstan. Can I say a-first, we support your war of terror! May we show our support to our boys in Iraq! May US and A kill every single terrorist! May your George Bush drink the blood of every single man, women, and child of Iraq! May you destroy their country so that for next thousand years not even a single lizard will survive in their desert!","japanese": "私の名前はボラット。カザフスタン出身です。まず最初に言っておきますが、私たちはあなたたちのテロ戦争を支持します！イラクにいる私たちの仲間たちにも支持を示せますように！アメリカとアメリカがすべてのテロリストを殺しますように！あなたたちのジョージ・ブッシュがイラクの男女すべての子供たちの血を飲みますように！あなたたちが彼らの国を滅ぼし、今後1000年間、彼らの砂漠でトカゲ一匹さえ生き残れないようにしますように！"}},{"_index":"nihongo_sensei","_id":"5O-oYpgBVKdRCT3-3Bhl","_score":1.0,"_source":{"@timestamp": "2025-07-31T21:53:11Z","english": "I am a cat. I still don't have a name. I don't have the slightest idea where I was born. But I do more or less remember the part where I was meowing and crying in some murky, damp place. It was there that I first saw what is known as a human.","japanese": "吾輩（わがはい）は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。何でも薄暗いじめじめした所でニャーニャー泣いていた事だけは記憶している。吾輩はここで始めて人間というものを見た。"}}]}}"#;

        // Set the response
        vm.set_response(200, json_response.to_string());

        // Verify that word wrap is disabled by default
        assert!(!vm.pane_manager().is_wrap_enabled(),
                "Word wrap should be disabled by default to prevent single-line responses from being broken");

        // Check the display cache to verify the response is treated as a single logical line
        let response_cache = vm
            .pane_manager()
            .get_display_cache(crate::repl::events::Pane::Response);

        if response_cache.is_valid {
            // The JSON response should be treated as one logical line (not wrapped)
            assert_eq!(
                response_cache.logical_to_display.len(),
                1,
                "JSON response should be treated as a single logical line"
            );

            // The single logical line should map to one display line
            if let Some(display_indices) = response_cache.logical_to_display.get(&0) {
                assert_eq!(
                    display_indices.len(),
                    1,
                    "Single logical line should map to one display line when wrap is disabled"
                );
            }
        }
    }

    #[test]
    fn test_wrap_can_still_be_enabled_if_needed() {
        // Create a ViewModel
        let mut vm = ViewModel::new();
        vm.update_terminal_size(80, 24);

        // Enable word wrap explicitly
        let _ = vm.set_wrap_enabled(true);

        // Verify wrap is now enabled
        assert!(
            vm.pane_manager().is_wrap_enabled(),
            "Word wrap should be enabled when explicitly set"
        );

        // Set a long response that would benefit from wrapping
        let long_response = "This is a very long line that should be wrapped when word wrap is enabled because it exceeds the terminal width and would be difficult to read on a single line.";
        vm.set_response(200, long_response.to_string());

        // Check that the response gets wrapped
        let response_cache = vm
            .pane_manager()
            .get_display_cache(crate::repl::events::Pane::Response);

        if response_cache.is_valid {
            // The long line should be wrapped into multiple display lines
            if let Some(display_indices) = response_cache.logical_to_display.get(&0) {
                assert!(display_indices.len() > 1,
                       "Long response should be wrapped into multiple display lines when wrap is enabled");
            }
        }
    }
}
