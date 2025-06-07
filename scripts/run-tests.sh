#!/bin/bash

# エラー発生時にスクリプト終了
set -e

# 色の設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=============================================${NC}"
echo -e "${BLUE}     MarkMail テスト実行スクリプト          ${NC}"
echo -e "${BLUE}=============================================${NC}"

# 作業ディレクトリをプロジェクトルートに設定
cd "$(dirname "$0")/.."
ROOT_DIR=$(pwd)

# バックエンドテスト実行
run_backend_tests() {
  echo -e "\n${YELLOW}バックエンドテスト開始...${NC}"
  
  cd "$ROOT_DIR/backend"
  
  # 環境変数が未設定の場合、テスト用の設定を行う
  if [ -z "$DATABASE_URL" ]; then
    echo -e "${YELLOW}DATABASE_URL環境変数を設定します...${NC}"
    export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/markmail"
    echo "DATABASE_URL=$DATABASE_URL"
  fi
  
  # テスト実行
  echo -e "${YELLOW}テスト実行中...${NC}"
  cargo test
  
  # 成功した場合
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}バックエンドテスト成功!${NC}"
    return 0
  else
    echo -e "${RED}バックエンドテスト失敗!${NC}"
    return 1
  fi
}

# フロントエンドテスト実行
run_frontend_tests() {
  echo -e "\n${YELLOW}フロントエンドテスト開始...${NC}"
  
  cd "$ROOT_DIR/frontend"
  
  # テスト実行
  echo -e "${YELLOW}テスト実行中...${NC}"
  npm test -- --run
  
  # 成功した場合
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}フロントエンドテスト成功!${NC}"
    return 0
  else
    echo -e "${RED}フロントエンドテスト失敗!${NC}"
    return 1
  fi
}

# 引数に応じてテスト実行
if [ "$1" == "backend" ]; then
  run_backend_tests
  exit $?
elif [ "$1" == "frontend" ]; then
  run_frontend_tests
  exit $?
else
  # 両方実行
  backend_result=0
  run_backend_tests || backend_result=$?

  frontend_result=0
  run_frontend_tests || frontend_result=$?

  # 結果の表示
  echo -e "\n${BLUE}=============================================${NC}"
  echo -e "${BLUE}               テスト結果                 ${NC}"
  echo -e "${BLUE}=============================================${NC}"

  if [ $backend_result -eq 0 ]; then
    echo -e "${GREEN}バックエンドテスト: 成功${NC}"
  else
    echo -e "${RED}バックエンドテスト: 失敗${NC}"
  fi

  if [ $frontend_result -eq 0 ]; then
    echo -e "${GREEN}フロントエンドテスト: 成功${NC}"
  else
    echo -e "${RED}フロントエンドテスト: 失敗${NC}"
  fi

  # 全体の結果
  if [ $backend_result -eq 0 ] && [ $frontend_result -eq 0 ]; then
    echo -e "\n${GREEN}すべてのテストが成功しました!${NC}"
    exit 0
  else
    echo -e "\n${RED}一部のテストが失敗しました!${NC}"
    exit 1
  fi
fi