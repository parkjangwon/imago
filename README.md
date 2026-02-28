# imago

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CLI](https://img.shields.io/badge/Type-CLI-blue)](#)

Rust 기반 CLI 이미지 생성기. Gemini Image Generation API를 사용해 프롬프트로 이미지를 만들고, 터미널 미리보기(선택)를 지원한다.

## 주요 기능
- 텍스트 프롬프트 기반 이미지 생성
- 파일 저장 경로 지정 지원
- 터미널 이미지 프리뷰 (`viuer`)
- 모델 404 대응 fallback 시도
- 단일 바이너리 CLI

## 요구사항
- Rust (stable)
- Gemini API Key

환경변수:
```bash
export GEMINI_API_KEY="your_api_key"
```

## 설치
```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/imago/main/install.sh | bash
```

## 빌드
```bash
cd /Users/pjw/dev/project/imago
cargo build --release
```

실행 파일:
```bash
./target/release/imago
```

## 사용법
```bash
imago "a cinematic blue cyberpunk city at night"
```

출력 경로 지정:
```bash
imago "minimal blue abstract geometric wallpaper" -o ./output/
```

미리보기 비활성화:
```bash
imago "product mockup on white desk" --no-preview
```

모델 지정(선택):
```bash
imago "futuristic interface concept" --model gemini-2.5-flash-image
```

## 옵션
```text
Usage: imago [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>                     생성할 이미지 설명

Options:
  -o, --output <PATH>          출력 파일 또는 디렉토리 경로
  -w, --width <COLUMNS>        터미널 프리뷰 너비 (기본: 60)
  -H, --height <ROWS>          터미널 프리뷰 높이 (선택)
      --no-preview             터미널 프리뷰 비활성화
  -m, --model <MODEL>          사용할 Gemini 모델
                                (기본: gemini-2.5-flash-image)
  -k, --api-key <KEY>          API 키 직접 지정 (환경변수보다 우선)
  -v, --verbose                상세 로그 출력
      --no-color               컬러 출력 비활성화
  -h, --help                   도움말
  -V, --version                버전
```

## 모델 fallback
기본 모델 요청이 404일 경우, 내부 fallback 모델을 순서대로 시도한다.
- gemini-2.5-flash-image
- gemini-3.1-flash-image-preview
- gemini-3-pro-image-preview
- gemini-2.0-flash-exp-image-generation

## 트러블슈팅
### 1) `GEMINI_API_KEY` 관련 오류
- 메시지: `API key not found`
- 조치: 환경변수 설정 확인 또는 `--api-key` 사용

### 2) 이미지 생성 실패(모델/권한)
- API Key 권한, 프로젝트 결제/쿼터, 모델 접근 가능 여부 확인
- `--model`로 명시 모델 지정 후 재시도

### 3) 터미널 프리뷰가 보이지 않음
- 사용하는 터미널의 이미지 프로토콜 지원 여부 확인
- 필요하면 `--no-preview`로 생성만 수행

## 라이선스
MIT
