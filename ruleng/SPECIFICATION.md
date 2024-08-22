# Ruleng Language Specification

## Overview

`ruleng` is a domain-specific language (DSL) designed to define rules for analyzing code changes and determining their potential impacts.
 It is used within the `impactifier` tool to parse, interpret, and execute rules provided by users, enabling automated impact analysis in various codebases.

## Unique Components

The `ruleng` language is composed of several key components:

- **Rules**: The top-level construct that encapsulates a specific analysis.
- **Triggers**: Define what changes should activate a rule.
- **Matchers**: Specify conditions that must be met for the rule to proceed.
- **Transformers**: Modify or prepare data before applying matchers.

## Syntax

The syntax of `ruleng` is designed to be human-readable and intuitive, with a structure that allows for both simple and complex rule definitions.

### Basic Structure

```ruleng
rule "RuleName" {
    trigger {
        ...
    }
    transform "TransformName" {
        ...
    }
    match {
        ...
    }
    action {
        ...
    }
}
```

### Components 

#### Rules

A rule is the highest-level construct in ruleng. It consists of a `trigger`, `match`, `transform`, and `action` block.

**Syntax:**
```ruleng
rule "RuleName" {
    trigger { ... }
    transform "TransformName" { ... }
    match { ... }
    action { ... }
}
```

**Example:**
```ruleng
rule "Detect API Changes" {
    trigger {
        path = "backend/**/*.go"
        match = regex("func (\\w+)Handler")
    }
    transform "toApiEndpoint" {
        input = "$1"
        steps = [
            { toLowerCase },
            { replace: { pattern: "Handler$", with: "_endpoint" } },
            { prepend: "/api/" }
        ]
        output = "$result"
    }
    match {
        path = "frontend/**/*.dart"
        match = regex("ApiClient.call('$transform')")
    }
    action {
        alert = Alert.Severe
    }
}
```

#### Triggers

Triggers define the conditions under which a rule is activated. They monitor changes in the codebase, such as modifications to specific files or patterns in code.

**Syntax:**
```ruleng
trigger {
    path = "file/path/pattern"
    match = regex("pattern")
}
```

#### Matchers

Matchers specify the conditions that must be met for a rule to proceed after a trigger is activated. They compare data against predefined patterns.

**Syntax:**
```ruleng
match {
    path = "file/path/pattern"
    match = regex("pattern")
}
```

#### Transformers

Transformers modify or prepare data before it is used in matchers or actions. They allow for complex transformations of data, such as converting function names to API endpoints.

```
transform "TransformName" {
    input = "$variable"
    steps = [
        { operation }
        ...
    ]
    output = "$result"
}
```

#### Actions

Actions define what should happen when a rule is triggered and its conditions are met

**Syntax:**
```ruleng
action {
    alert = "Alert message"
    generate_report = "report/file/path"
}
```

#### Variable Declaration

**Syntax:**

```ruleng
let VariableName = value;
```

**Example:**
```ruleng
let ApiPattern = "func (\\w+)Handler";
let ApiTransformer = transform "ApiTransformer" {
    input = "$1"
    steps = [
        { toLowerCase },
        { replace: { pattern: "Handler$", with: "_endpoint" } },
        { prepend: "/api/" }
    ]
    output = "$result"
};
```

#### Imports

**Syntax:**
```ruleng
import "filepath"; // `.ruleng` is not needed, as it is the only file extension that can be imported 
```

#### Conditions 

Only simplest `if`, `else if`, `else` is supported.

**Syntax:**

```ruleng
if condition {
   
} else {

}
```

#### Loops

Only `for in` loop is supported.

**Syntax:**
```ruleng
for (item in collection) {
    // statements
}
```

**Example:**

```ruleng
let files = ["file1.dart", "file2.dart", "file3.dart"];

for (file in files) {
    rule "Check File" {
        trigger {
            path = file
        }
        match {
            match = regex("ApiClient.call")
        }
        alert = Alert.Low;
    }
}
```

#### Functions

Only positional parameters are supported.

**Syntax:**
```ruleng
fn FunctionName(parameter1, parameter2, ...) {
    // function body
    return result;
}
```

#### Enums

```ruleng
enum AlertLevel {
    Low,
    Moderate,
    High,
    Severe
}
```

####  Operators

1. **Arithmetic Operators**
    - **`+`**: Addition or concatenation.
    - **`-`**: Subtraction.
    - **`*`**: Multiplication.
    - **`/`**: Division.
    - **`%`**: Modulus (remainder of division).

    **Examples:**
    ```ruleng
    let sum = 5 + 3;
    let difference = 10 - 2;
    let product = 4 * 7;
    let quotient = 20 / 4;
    let remainder = 10 % 3;
    ```

    **Concatenation Example:**
    ```ruleng
    let fullName = "John" + " " + "Doe";
    ```

2. **Comparison Operators**
    - **`==`**: Equal to.
    - **`!=`**: Not equal to.
    - **`<`**: Less than.
    - **`<=`**: Less than or equal to.
    - **`>`**: Greater than.
    - **`>=`**: Greater than or equal to.

    **Examples:**
    ```ruleng
    let isEqual = (5 + 3) == 8;
    let isNotEqual = (10 - 2) != 5;
    let isLessThan = 3 < 5;
    let isLessThanOrEqual = 7 <= 7;
    let isGreaterThan = 10 > 6;
    let isGreaterThanOrEqual = 15 >= 10;
    ```

3. **Logical Operators**
    - **`&&`**: Logical AND.
    - **`||`**: Logical OR.
    - **`!`**: Logical NOT.

    **Examples:**
    ```ruleng
    let andCondition = (5 > 3) && (8 > 6);
    let orCondition = (5 < 3) || (8 > 6);
    let notCondition = !(5 == 3);
    ```