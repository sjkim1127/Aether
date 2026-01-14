// Test file for @aether/codegen
// Run with: node test.js

const { Template, AetherEngine, generate, renderTemplate } = require('./index');

async function main() {
    console.log('🚀 @aether/codegen - Node.js Bindings Test\n');

    // Test 1: Template creation
    console.log('Test 1: Template creation');
    const template = new Template('<div>{{AI:content}}</div>');
    template.setSlot('content', 'Generate a welcome message');
    console.log('  ✅ Template created');
    console.log('  Slots:', template.getSlotNames());

    // Test 2: Engine creation
    console.log('\nTest 2: Engine creation');
    const engine = AetherEngine.openai('gpt-5.2-thinking');
    console.log('  ✅ OpenAI engine created');

    const anthropicEngine = AetherEngine.anthropic();
    console.log('  ✅ Anthropic engine created');

    const ollamaEngine = AetherEngine.ollama('codellama');
    console.log('  ✅ Ollama engine created');

    // Test 3: One-line generation (requires API key)
    console.log('\nTest 3: Code generation');
    if (process.env.OPENAI_API_KEY) {
        try {
            const code = await generate('Create a simple HTML button');
            console.log('  ✅ Generated code:', code.substring(0, 50) + '...');
        } catch (e) {
            console.log('  ⚠️ Generation failed:', e.message);
        }
    } else {
        console.log('  ⏭️ Skipped (OPENAI_API_KEY not set)');
    }

    console.log('\n✅ All tests completed!');
}

main().catch(console.error);
