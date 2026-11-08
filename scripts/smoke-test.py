#!/usr/bin/env python3
"""Exercise a running API, including authentication and cross-user isolation."""
import json
import os
import urllib.error
import urllib.request
import uuid

BASE = os.environ.get('API_URL', 'http://localhost:8080').rstrip('/')

def request(method, path, body=None, token=None, expected=200):
    headers = {'Content-Type': 'application/json'}
    if token:
        headers['Authorization'] = 'Bearer ' + token
    req = urllib.request.Request(BASE + path, data=json.dumps(body).encode() if body is not None else None, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=15) as response:
            status, data = response.status, response.read()
    except urllib.error.HTTPError as error:
        status, data = error.code, error.read()
    assert status == expected, (method, path, status, expected, data.decode())
    return json.loads(data) if data else None

suffix = uuid.uuid4().hex
request('GET', '/health')
request('GET', '/api/todos', expected=401)
accounts = []
for name in ('Alice', 'Bob'):
    body = {'name': name, 'email': f'{name.lower()}-{suffix}@example.com', 'password': 'test-password-123'}
    accounts.append((body, request('POST', '/api/auth/signup', body, expected=201)))
body, alice = accounts[0]
bob = accounts[1][1]
request('POST', '/api/auth/signup', body, expected=409)
request('POST', '/api/auth/login', {'email': body['email'], 'password': 'wrong'}, expected=401)
logged_in = request('POST', '/api/auth/login', {'email': body['email'], 'password': body['password']})
assert request('GET', '/api/auth/me', token=logged_in['token'])['id'] == alice['user']['id']
request('POST', '/api/todos', {'title': '   '}, alice['token'], expected=400)
todo = request('POST', '/api/todos', {'title': '  Verify full stack  '}, alice['token'], expected=201)
assert todo['title'] == 'Verify full stack'
assert any(item['id'] == todo['id'] for item in request('GET', '/api/todos', token=alice['token']))
assert not request('GET', '/api/todos', token=bob['token'])
path = '/api/todos/' + todo['id']
request('PATCH', path, {'completed': True}, bob['token'], expected=404)
request('DELETE', path, token=bob['token'], expected=404)
updated = request('PATCH', path, {'title': 'Verified', 'completed': True}, alice['token'])
assert updated['title'] == 'Verified' and updated['completed']
request('DELETE', path, token=alice['token'], expected=204)
request('PATCH', path, {'completed': False}, alice['token'], expected=404)
request('GET', '/api/todos', token='invalid', expected=401)
print('PASS: health, signup, login, validation, CRUD, invalid tokens, and cross-user isolation')
