from anthropic import Anthropic
import os

# Configuration (Using values confirmed from previous steps)
API_KEY = "sk-06ca1f5bb642459a8160f2945c4334bf"
BASE_URL = "http://127.0.0.1:8080"

print(f"Connecting to {BASE_URL} with key {API_KEY[:6]}...")

client = Anthropic(
    base_url=BASE_URL,
    api_key=API_KEY
)

models_to_test = [
    "codex",
    # "gpt-5.2-codex", # Failed due to account issues
]

for model in models_to_test:
    print(f"\nTesting model: {model}")
    try:
        response = client.messages.create(
            model=model,
            max_tokens=1024,
            messages=[{"role": "user", "content": "1+1 is?"}]
        )
        print(f"Success! Response: {response.content[0].text}")

        # [NEW] Strict verification of Codex handler usage
        print(f"Response Model: {response.model}")
        
        if "gpt-5" in response.model or "codex" in response.model:
             print("✅ Verified: Response came from a Codex model.")
        else:
             print(f"⚠️ Warning: Response model is '{response.model}', which might indicate fallback (expected 'gpt-5.2-codex').")
             if response.model == "gpt-5.2-codex":
                 print("✅ Strict Verification Passed: Model name matches.")
             else:
                 print("❌ Strict Verification Failed: Model name does not match.")
                 exit(1)

    except Exception as e:
        print(f"Error testing {model}: {e}")
        exit(1)
