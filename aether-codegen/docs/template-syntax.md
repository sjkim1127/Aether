# Template Syntax

Aether uses a simple template syntax for defining AI injection points.

## Basic Syntax

```
{{AI:slot_name}}
```

This creates a slot named `slot_name` that will be replaced with AI-generated code.

## Slot Kinds

You can specify the type of code to generate:

```
{{AI:slot_name:kind}}
```

### Available Kinds

| Kind | Syntax | Description |
|------|--------|-------------|
| Raw | `{{AI:name}}` | Raw code injection (default) |
| HTML | `{{AI:name:html}}` | Valid HTML5 markup |
| CSS | `{{AI:name:css}}` | CSS styles |
| JavaScript | `{{AI:name:js}}` | JavaScript code |
| Function | `{{AI:name:function}}` | Function definition |
| Class | `{{AI:name:class}}` | Class/struct definition |
| Component | `{{AI:name:component}}` | Full component (HTML+CSS+JS) |

## Examples

### Simple Template

```html
<h1>{{AI:title}}</h1>
```

### Web Page Template

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        {{AI:styles:css}}
    </style>
</head>
<body>
    <header>{{AI:header:html}}</header>
    <main>{{AI:content:html}}</main>
    <footer>{{AI:footer:html}}</footer>
    <script>
        {{AI:script:js}}
    </script>
</body>
</html>
```

### React Component Template

```jsx
import React from 'react';

{{AI:imports:raw}}

export function {{AI:component_name:raw}}(props) {
    {{AI:state:raw}}

    {{AI:handlers:function}}

    return (
        {{AI:jsx:html}}
    );
}
```

### Rust Template

```rust
{{AI:imports:raw}}

/// {{AI:doc_comment:raw}}
pub struct {{AI:struct_name:class}} {
    {{AI:fields:raw}}
}

impl {{AI:struct_name:raw}} {
    {{AI:methods:function}}
}
```

## Naming Rules

Slot names must:
- Start with a letter or underscore
- Contain only letters, numbers, and underscores
- Be unique within a template

**Valid**: `content`, `button_text`, `_private`, `section1`
**Invalid**: `1section`, `my-slot`, `slot.name`

## Escaping

To include literal `{{AI:...}}` in your output without replacement, you can:
1. Use a placeholder slot with a literal value
2. Process the template in multiple passes

## Best Practices

1. **Descriptive Names**: Use clear slot names like `login_form` instead of `form1`
2. **Specify Kinds**: Always specify the kind when you know the expected output type
3. **Keep Slots Focused**: Each slot should have a single responsibility
4. **Use Context**: Provide surrounding code in the template for better AI context
