# Raw AI API vs. Aether Codegen

When building AI-powered applications, developers often start with raw API calls (e.g., using the OpenAI SDK). While this works for simple chatbots, it quickly becomes unmanageable for **structured code generation**. 

Here is why Aether is the superior choice for production-grade AI injection.

## Feature Comparison

| Feature | Raw AI API Call | Aether Codegen |
| :--- | :--- | :--- |
| **Code Structure** | String concatenation & messy Regex parsing. | **Template & Slot-based**. Precise injection. |
| **Reliability** | "Fire and forget." No way to know if code works. | **Self-Healing**. Auto-validates & fixes code loops. |
| **Performance** | Repeated calls for the same prompt cost money/time. | **Semantic Caching**. 0.1ms responses for similar tasks. |
| **Token Cost** | Naive prompt sending. High overhead. | **TOON Protocol**. Optimized for minimal token usage. |
| **Developer DX** | Handling multiple SDKs (OpenAI, Claude, etc). | **Uniform Interface**. Switch providers with one line. |
| **Creativity** | Global temperature for the whole response. | **Per-Slot Temperature**. Creative text + strict logic. |

---

## Code Comparison (Real-world Scenario)

### ❌ The Raw API Way (Messy)
```javascript
// You have to manually manage the prompt, parse the results, 
// and handle the risk of AI returning markdown blocks or explanations.
const response = await openai.chat.completions.create({
  model: "gpt-4",
  messages: [{ role: "user", content: "Give me only the body of a rust function that..." }]
});

let code = response.choices[0].message.content;
// Manual sanitization needed
code = code.replace(/```rust/g, "").replace(/```/g, ""); 

// If there's a syntax error, your app just crashes or fails at runtime.
const finalFile = `fn my_func() {\n${code}\n}`;
```

### ✅ The Aether Way (Clean & Robust)
```javascript
// Aether handles parsing, sanitization, and reliability automatically.
const template = new Template("fn my_func() { {{AI:logic}} }");

template.setSlot("logic", "Implement safe recursion", 0.0); // Strict!

// engine.setHeal(true) ensures the code actually compiles before returning.
const result = await engine.render(template);
```

---

## Why it Matters

### 1. The "Hallucination" Tax
Raw APIs frequently hallucinate non-existent libraries or forget a semicolon. Without Aether's **Healing** loop, you are passing those hallucinations directly to your users or your build system.

### 2. The "Latency" Tax
Calling an LLM takes seconds. If 5 users ask for the same "Sorting Algorithm explanation," raw APIs charge you 5 times and make 5 users wait. Aether's **Semantic Cache** serves it from local memory after the first call.

### 3. The "Vendor Lock-in" Tax
Switching from OpenAI to Claude with raw SDKs requires rewriting your networking and error-handling logic. In Aether, it's a simple factory swap: `AetherEngine.openai()` -> `AetherEngine.anthropic()`.

---

## Conclusion

Raw API calls are for **conversations**. Aether is for **engineering**. 

If your AI output needs to be part of a reliable software system, don't just call an API—**orchestrate it with Aether.**
