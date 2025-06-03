// マークダウン処理サービス
// TODO: マークダウンからHTMLへの変換、変数抽出の実装

use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use serde_json::Value;

pub struct MarkdownService;

impl MarkdownService {
    pub fn new() -> Self {
        Self
    }

    /// マークダウンをHTMLに変換
    pub fn render_to_html(&self, markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // メール用のスタイリングを追加
        self.add_email_styles(&html_output)
    }

    /// テンプレート変数を置換
    pub fn render_with_variables(
        &self,
        markdown: &str,
        variables: &Value,
    ) -> Result<String, String> {
        let substituted_markdown = self.substitute_variables(markdown, variables)?;
        Ok(self.render_to_html(&substituted_markdown))
    }

    /// テンプレート変数の置換
    fn substitute_variables(&self, content: &str, variables: &Value) -> Result<String, String> {
        let re =
            Regex::new(r"\{\{\s*(\w+)\s*\}\}").map_err(|e| format!("正規表現エラー: {}", e))?;

        let mut result = content.to_string();

        for cap in re.captures_iter(content) {
            let variable_name = &cap[1];
            let placeholder = &cap[0];

            if let Some(value) = variables.get(variable_name) {
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(placeholder, &replacement);
            } else {
                return Err(format!("変数 '{}' が見つかりません", variable_name));
            }
        }

        Ok(result)
    }

    /// テンプレートから変数を抽出
    pub fn extract_variables(&self, content: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();
        let mut variables = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cap in re.captures_iter(content) {
            let variable_name = cap[1].to_string();
            if seen.insert(variable_name.clone()) {
                variables.push(variable_name);
            }
        }

        variables
    }

    /// メール用のスタイリングを追加
    fn add_email_styles(&self, html: &str) -> String {
        let email_css = r#"
<style>
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        line-height: 1.6;
        color: #333;
        max-width: 600px;
        margin: 0 auto;
        padding: 20px;
    }
    h1, h2, h3, h4, h5, h6 {
        color: #2c3e50;
        margin-top: 30px;
        margin-bottom: 15px;
    }
    h1 { font-size: 2.2em; border-bottom: 2px solid #3498db; padding-bottom: 10px; }
    h2 { font-size: 1.8em; border-bottom: 1px solid #ecf0f1; padding-bottom: 8px; }
    h3 { font-size: 1.4em; }
    p { margin-bottom: 16px; }
    a { color: #3498db; text-decoration: none; }
    a:hover { text-decoration: underline; }
    blockquote {
        border-left: 4px solid #3498db;
        margin: 20px 0;
        padding: 10px 20px;
        background-color: #f8f9fa;
        font-style: italic;
    }
    code {
        background-color: #f1f2f6;
        padding: 2px 6px;
        border-radius: 3px;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.9em;
    }
    pre {
        background-color: #f8f9fa;
        border: 1px solid #e9ecef;
        border-radius: 5px;
        padding: 15px;
        overflow-x: auto;
        margin: 20px 0;
    }
    pre code {
        background-color: transparent;
        padding: 0;
    }
    table {
        border-collapse: collapse;
        width: 100%;
        margin: 20px 0;
    }
    th, td {
        border: 1px solid #ddd;
        padding: 12px;
        text-align: left;
    }
    th {
        background-color: #f2f2f2;
        font-weight: bold;
    }
    ul, ol {
        margin: 16px 0;
        padding-left: 30px;
    }
    li {
        margin-bottom: 8px;
    }
    .button {
        display: inline-block;
        padding: 12px 24px;
        background-color: #3498db;
        color: white !important;
        text-decoration: none;
        border-radius: 5px;
        margin: 10px 0;
    }
    .button:hover {
        background-color: #2980b9;
        text-decoration: none;
    }
</style>
"#;

        format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<meta name=\"viewport\" \
             content=\"width=device-width, \
             initial-scale=1.0\">\n{}\n</head>\n<body>\n{}\n</body>\n</html>",
            email_css, html
        )
    }

    /// マークダウンの構文チェック
    pub fn validate_markdown(&self, markdown: &str) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        // 基本的な構文チェック
        if markdown.trim().is_empty() {
            errors.push("マークダウンコンテンツが空です".to_string());
        }

        // 未閉じのコードブロックをチェック
        let code_block_count = markdown.matches("```").count();
        if code_block_count % 2 != 0 {
            errors.push("コードブロックが正しく閉じられていません".to_string());
        }

        // リンクの構文チェック
        let link_regex = Regex::new(r"\[([^\]]*)\]\(([^\)]*)\)")
            .map_err(|e| format!("正規表現エラー: {}", e))?;

        for cap in link_regex.captures_iter(markdown) {
            let url = &cap[2];
            if url.trim().is_empty() {
                errors.push(format!("空のURLが見つかりました: [{}]()", &cap[1]));
            }
        }

        Ok(errors)
    }

    /// 件名テンプレートを変数で置換
    pub fn render_subject(
        &self,
        subject_template: &str,
        variables: &Value,
    ) -> Result<String, String> {
        self.substitute_variables(subject_template, variables)
    }
}
