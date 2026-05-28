; zsheesh formatting rules for zsh
; Based on Topiary's bash formatting queries, extended for zsh constructs.
; Uses tree-sitter-zsh (forked from tree-sitter-bash, extended with zsh syntax).

; Leaves: nodes whose content must not be reformatted.
[
  (comment)
  (expansion)
  (heredoc_redirect)
  (string)
  (word)
] @leaf

(simple_expansion
  "$"
  .
  (_) @leaf
)

(subscript
  index: (_) @leaf
)

;; Spacing

; Allow blank line before
[
  (c_style_for_statement)
  (case_item)
  (case_statement)
  (command)
  (comment)
  (compound_statement)
  (declaration_command)
  (for_statement)
  (function_definition)
  (if_statement)
  (list)
  (pipeline)
  (redirected_statement)
  (subshell)
  (variable_assignment)
  (while_statement)
] @allow_blank_line_before

; Insert a new line before multi-line syntactic blocks
[
  (c_style_for_statement)
  (case_statement)
  (for_statement)
  (function_definition)
  (if_statement)
  (while_statement)
] @prepend_hardline

; Subshells and compound statements get a new line only at the top level
(program
  [
    (compound_statement)
    (subshell)
  ] @prepend_hardline
)

; Interpose a new line between multi-line blocks and commands
(
  [
    (c_style_for_statement)
    (case_statement)
    (declaration_command)
    (for_statement)
    (function_definition)
    (if_statement)
    (variable_assignment)
    (while_statement)
  ]
  .
  [(command) (list) (pipeline) (subshell) (compound_statement) (redirected_statement)] @prepend_hardline
)

; Interpose a new line before variable declarations after other constructs
(
  [
    (c_style_for_statement)
    (case_statement)
    (command)
    (compound_statement)
    (for_statement)
    (function_definition)
    (if_statement)
    (list)
    (pipeline)
    (redirected_statement)
    (subshell)
    (while_statement)
  ]
  .
  [
    (declaration_command)
    (variable_assignment)
  ] @prepend_hardline
)

; Append a space to keywords and delimiters
[
  ";"
  "case"
  "declare"
  "do"
  "elif"
  "export"
  "for"
  "if"
  "in"
  "local"
  "readonly"
  "select"
  "then"
  "typeset"
  "until"
  "while"
] @append_space

";" @prepend_antispace

; Prepend a space to intra-statement keywords
[
  "in"
] @prepend_space

;; Comments

(comment) @append_hardline
(comment) @prepend_space

;; Compound Statements and Subshells

(compound_statement
  .
  "{" @append_spaced_softline @append_indent_start
  _
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

(subshell
  .
  "(" @append_spaced_softline @append_indent_start
  _
  ")" @prepend_spaced_softline @prepend_indent_end
  .
)

;; Commands — line breaks in various contexts

(program
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "program_line_break")
)

(compound_statement
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "compound_statement_line_break")
)

(subshell
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "subshell_line_break")
)

(if_statement
  .
  _
  "then"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "if_statement_line_break")
)

(elif_clause
  .
  _
  "then"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "elif_clause_line_break")
)

(else_clause
  .
  "else"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "else_clause_line_break")
)

(case_item
  .
  _
  ")"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "case_item_line_break")
)

(do_group
  .
  "do"
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "do_group_line_break")
)

(command_substitution
  [
    (command)
    (list)
    (pipeline)
    (subshell)
    (compound_statement)
    (redirected_statement)
    (variable_assignment)
  ] @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope
  (#scope_id! "command_substitution_line_break")
)

; Spaces between list/pipeline delimiters
(list
  [(_) "&&" "||"] @append_space
  .
  _
)

(pipeline
  ["|" "|&"] @prepend_space @append_spaced_softline
)

(pipeline
  .
  (_)
  .
  ["|" "|&"] @append_indent_start
) @append_indent_end

; Asynchronous operator
(_
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement)]
  .
  "&" @prepend_space @append_spaced_softline
)

; Spaces between command and its arguments
(command
  (_) @append_space
  .
  (_)
)

; Negation operator
(negated_command
  .
  "!" @prepend_space @append_space
)

; Backtick command substitution to $() form
(command_substitution
  .
  "`" @delete @append_delimiter
  .
  _ @prepend_empty_softline @prepend_indent_start
  (#delimiter! "$(")
)

(command_substitution
  _ @append_empty_softline @append_indent_end
  .
  "`" @delete @append_delimiter
  .
  (#delimiter! ")")
)

; Multi-line command substitutions become an indent block
(command_substitution
  "$(" @append_empty_softline @append_indent_start
  ")" @prepend_empty_softline @prepend_indent_end
)

; Space interposes command substitutions containing subshells
(command_substitution
  .
  (subshell) @prepend_space
)

(command_substitution
  (subshell) @append_space
  .
)

;; Redirections

(redirected_statement
  (_) @append_space
  .
  (_)
)

(herestring_redirect (_) @prepend_space)

;; Conditionals

[
  (if_statement)
  (elif_clause)
  (else_clause)
] @append_hardline

[
  (if_statement)
  (elif_clause)
]
"then" @append_hardline @append_indent_start

(else_clause
  .
  "else" @append_hardline @append_indent_start
)

(if_statement
  [
    "fi"
    (else_clause)
    (elif_clause)
  ] @prepend_indent_end @prepend_hardline
)

; Keep "if"/"elif" and "then" on the same line
(_
  ";"* @do_nothing
  .
  "then" @prepend_delimiter
  (#delimiter! "; ")
)

;; Test Commands

(test_command
  "[" @append_space
  "]" @prepend_space
)

(test_command
  "[[" @append_space
  "]]" @prepend_space
)

(arithmetic_expansion
  "$[" @delete @append_delimiter
  (#delimiter! "$(( ")
)

(arithmetic_expansion
  "]" @delete @append_delimiter
  (#delimiter! " ))")
)

(arithmetic_expansion
  ["$((" "(("] @append_space
  ["))"] @prepend_space
)

(unary_expression
  "!" @append_space
)

(unary_expression
  (test_operator) @append_space
)

(binary_expression
  left: (_) @append_space
  right: (_) @prepend_space
)

;; Case Statements

(case_statement
  .
  "case"
  .
  _
  .
  "in" @append_hardline @append_indent_start
  _
  "esac" @prepend_hardline @prepend_indent_end
  .
) @append_hardline

(case_item
  ")" @append_hardline @append_indent_start
) @append_indent_end

(case_item
  [
    ";;"
    ";;&"
    ";&"
  ] @append_hardline
  .
)

;; Loops

(do_group
  .
  "do" @append_hardline @append_indent_start
  _
  "done" @prepend_hardline @prepend_indent_end
  .
) @append_hardline

(for_statement
  value: _* @prepend_space
)

(c_style_for_statement
  initializer: _ @prepend_space
  update: _ @append_space
)

; Keep loop construct and "do" on the same line
(_
  ";"* @do_nothing
  .
  (do_group) @prepend_delimiter
  (#delimiter! "; ")
)

;; Function Definitions

(function_definition
  body: (_) @prepend_space @append_hardline
)

(function_definition
  .
  (word) @append_delimiter
  .
  (
    "("
    ")"
  )? @do_nothing

  (#delimiter! "()")
)

(function_definition
  .
  "function" @delete
)

;; Variable Declaration, Assignment and Expansion

(
  (declaration_command)
  .
  (_) @prepend_hardline
)

(declaration_command
  (word) @append_space
)

(declaration_command
  [
    (variable_name)
    (variable_assignment)
  ] @prepend_space
)

(declaration_command
  (variable_name)? @do_nothing
  .
  (concatenation) @prepend_space
)

(command
  (variable_assignment) @append_space
)

; Multi-line arrays start an indentation block
(array
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_empty_softline @prepend_indent_end
)

(array
  (_) @append_spaced_softline
  .
  (_)
)

; Convert (simple_expansion) into (expansion)s
(simple_expansion
  (#delimiter! "{")
  "$"
  .
  (variable_name) @prepend_delimiter
  (#not-match? @prepend_delimiter "[0-9]")
)

(simple_expansion
  (#delimiter! "}")
  "$"
  .
  (variable_name) @append_delimiter
  (#not-match? @append_delimiter "[0-9]")
)
