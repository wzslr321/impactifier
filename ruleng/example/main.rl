-- Single-line comment
/* 
    Multi-line comment
*/

-- Functions are most useful for transformers.
-- Functions support only positional, untyped parameters.
func toUpperCase(value) {
    if isString(value) {
        return value.toUpperCase()
    }
    -- Expression inside a string can be used via ${} syntax.
    -- Error is the only type that can be used along `throw`.
    -- Custom error types are not supported.
    -- Error message will always include the stacktrace and its 
    -- position in the code, with message specified by user appended.
    throw Error("String expected, but received ${typeof(value)}")
}

-- Global variables
let API_PATH = "api/"
let CLIENT_PATH = "client/"

rule "Detect API Changes" {
    -- Trigger defines what activates the rule
    trigger {
        -- Path in which to look for changes, along with file type specification
        path = $API_PATH
        -- (Optional) What exact changes trigger the rule
        -- Example below matches everything that affects functions, which name ends with 'Handler'
        pattern = regex("func (\\w+)Handler")
        -- (Optional) defaults to false
        -- Specifies if changes to dependencies of matched pattern are a trigger as well.
        -- e.g., when set to true, if implementation of a function executed inside the handler changed, 
        -- but the handler itself did not, the trigger will still be activated
        analyze_dependencies = true
    }

    -- Optional
    -- How to transform trigger matched in `trigger` property
    -- in order to properly look for its usages in another part of the codebase.
    transform = "toApiEndpoint" {
        input = "$1"
        steps = [
            { toLowerCase },
            { replace(pattern: "Handler$", with: "_endpoint") },
            { prepend("/api/") }
        ]
        output = "$result"
    }

    -- What triggers the rule action.
    -- In this case, it takes the output of `transform` and checks
    -- if there is a specified pattern inside $CLIENT_PATH
    match {
        path = $CLIENT_PATH
        pattern = regex("ApiClient.call('$transform')")
    }
    -- If the rule is satisfied by the trigger itself, and no further matching is needed,
    -- it can be specified via `always` keyword:
    --
    -- match = always

    -- Executed when rule's `match` returned true
    action {
        alert = Alert.Severe
        message = "API changed"
        -- Specify report generation parameters
        report {
            format = "json"
            destination = "./reports/api_changes_report.json"
        }
    }
}

