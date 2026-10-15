# rusty-book-manager

書籍『[Rust による Web アプリケーション開発 設計からリリース・運用まで](https://www.kspub.co.jp/book/detail/5369579.html)』のサンプルアプリケーションを、学習目的で実装したリポジトリです。

- 書籍: <https://www.kspub.co.jp/book/detail/5369579.html>
- 本家リポジトリ: <https://github.com/rust-web-app-book/rusty-book-manager>

## 構成

- `backend/` — Rust (axum / sqlx / PostgreSQL / Redis) の REST API
- `frontend/` — SvelteKit 3 (Svelte 5 + TypeScript) の SPA。pnpm + [Vite+](https://viteplus.dev/) (Vite / Vitest / Oxlint / Oxfmt)

## 開発

必要なツールは [mise](https://mise.jdx.dev/) で管理しています (`mise.toml`)。タスクは
`back-*` (バックエンド) / `front-*` (フロントエンド) / プレフィックスなし (共有インフラと
リポジトリ全体) の 3 つに分かれています。一覧は `mise tasks` で確認できます。

### バックエンド

```sh
mise run back-run      # PostgreSQL / Redis / Jaeger を起動し、マイグレーション後に API を起動
mise run back-seed     # 初期データ (管理者ユーザー) を投入
mise run back-test     # テスト
mise run back-ci       # fmt / clippy / test
```

API は `http://localhost:8080` で待ち受けます。OpenAPI は開発ビルドでのみ
`/swagger-ui`、`/docs`、`/api-docs/openapi.json` で参照できます。

### フロントエンド

```sh
mise run front-install # pnpm install
mise run front-dev     # http://localhost:5173
mise run front-check   # フォーマット / lint / 型チェック
mise run front-test    # Vitest
mise run front-build   # ビルド (adapter-node)
mise run front-serve   # ビルドして http://localhost:3000 で配信
```

### 全体

```sh
mise run ci            # back-ci + front-ci
mise run up            # PostgreSQL / Redis / Jaeger を起動 (down / reset / logs も同様)
```

詳細は [`frontend/README.md`](frontend/README.md) を参照してください。
初期データの管理者は `eleazar.fig@example.com` / `password` です。

## ライセンス

MIT
