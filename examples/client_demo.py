"""
Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

PaySwitch Python Integration Client
Demonstrates cross-border payment switching and balance query.
"""

import json
import urllib.request

BASE_URL = "http://127.0.0.1:8080"

def check_health():
    req = urllib.request.Request(f"{BASE_URL}/health")
    with urllib.request.urlopen(req) as res:
        data = json.loads(res.read().decode())
        print(f"Node Status: {data['data']['status']} (Version {data['data']['version']})")

def create_account(account_type: str, currency: str) -> str:
    payload = json.dumps({"account_type": account_type, "currency": currency}).encode()
    req = urllib.request.Request(
        f"{BASE_URL}/api/v1/accounts",
        data=payload,
        headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req) as res:
        data = json.loads(res.read().decode())
        return data["data"]

def get_account_balance(account_id: str):
    req = urllib.request.Request(f"{BASE_URL}/api/v1/accounts/{account_id}")
    with urllib.request.urlopen(req) as res:
        data = json.loads(res.read().decode())
        balance = data["data"]["settled_balance"]
        print(f"Account {account_id} Balance: {balance} atomic units")

if __name__ == "__main__":
    try:
        check_health()
    except Exception as e:
        print(f"Node connection failed: {e}. Ensure 'payswitch-node' is running.")
