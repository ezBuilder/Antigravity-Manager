"""PM 라우터 테스트 - HTTP 요청만 사용"""
import json
import urllib.request
import time

BASE_URL = "http://127.0.0.1:8045"

def test_health():
    """헬스 체크"""
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/health", method="GET")
        with urllib.request.urlopen(req, timeout=5) as resp:
            if resp.status == 200:
                print("✅ Health check: OK")
                return True
            else:
                print(f"❌ Health check failed: {resp.status}")
                return False
    except Exception as e:
        print(f"❌ Proxy unreachable: {e}")
        return False

def test_models():
    """모델 목록 조회"""
    try:
        req = urllib.request.Request(f"{BASE_URL}/v1/models", method="GET")
        req.add_header("Authorization", "Bearer sk-test")
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode())
            models = [m["id"] for m in data.get("data", [])]
            print(f"✅ Models list: {len(models)} models")
            print(f"   Available: {', '.join(models[:5])}...")
            return True
    except Exception as e:
        print(f"❌ Models list error: {e}")
        return False

def test_chat(model="gemini-3-flash", test_name="Basic Chat"):
    """채팅 테스트"""
    print(f"\n🧪 Testing {test_name} (model: {model})...")
    try:
        body = {
            "model": model,
            "messages": [{"role": "user", "content": "Hello, respond with just 'Hi'"}],
            "max_tokens": 10
        }

        req = urllib.request.Request(
            f"{BASE_URL}/v1/chat/completions",
            data=json.dumps(body).encode(),
            method="POST"
        )
        req.add_header("Authorization", "Bearer sk-test")
        req.add_header("Content-Type", "application/json")

        start_time = time.time()
        with urllib.request.urlopen(req, timeout=30) as resp:
            duration = time.time() - start_time
            data = json.loads(resp.read().decode())

            content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
            used_model = data.get("model", "unknown")

            print(f"✅ Chat response ({duration:.2f}s)")
            print(f"   Requested: {model}")
            print(f"   Used: {used_model}")
            print(f"   Response: {content[:100]}")

            return True, used_model
    except urllib.error.HTTPError as e:
        error_body = e.read().decode()
        print(f"❌ Chat error {e.code}: {error_body}")
        return False, None
    except Exception as e:
        print(f"❌ Chat error: {e}")
        return False, None

def main():
    print("=" * 60)
    print("PM 라우터 테스트")
    print("=" * 60)

    # 1. Health check
    if not test_health():
        print("\n⚠️ 프록시가 실행되지 않았습니다. Antigravity Manager를 먼저 실행해주세요.")
        return

    print()

    # 2. Models list
    if not test_models():
        return

    # 3. Chat tests
    print("\n" + "=" * 60)
    print("채팅 테스트 - PM 라우터 동작 확인")
    print("=" * 60)

    # Test 1: Gemini Flash (simple task)
    test_chat("gemini-3-flash", "Simple Task (Gemini Flash)")
    time.sleep(1)

    # Test 2: Codex model (코딩 관련)
    success, used = test_chat("gpt-5.2-codex", "Coding Task (Codex)")
    if success and used:
        if "codex" in used.lower():
            print("\n🎉 PM 라우터가 Codex 모델을 정상적으로 사용했습니다!")
        else:
            print(f"\n⚠️ PM 라우터가 Codex 대신 {used} 모델을 사용했습니다.")

    print("\n" + "=" * 60)
    print("테스트 완료")
    print("=" * 60)

if __name__ == "__main__":
    main()
