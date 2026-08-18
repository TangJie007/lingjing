import json, re

def find_ids(obj, path=''):
    out = []
    if isinstance(obj, dict):
        for k, v in obj.items():
            out += find_ids(v, path + '.' + k)
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            out += find_ids(v, path + f'[{i}]')
    elif isinstance(obj, str):
        for m in re.findall(r'\b(A\d+)\b', obj):
            out.append((path, 'A' + m))
    return out

d = json.load(open('st.json', encoding='utf-16'))
res = find_ids(d)
uniq = sorted(set(x[1] for x in res), key=lambda x: int(x[1:]))
print('total found:', len(uniq))
print('first:', uniq[:8])
print('last:', uniq[-4:])
# also look for any 'acceptance' array with id field
def find_acc_arrays(obj):
    out = []
    if isinstance(obj, dict):
        if 'acceptance' in obj and isinstance(obj['acceptance'], list):
            out.append(obj['acceptance'])
        for v in obj.values():
            out += find_acc_arrays(v)
    elif isinstance(obj, list):
        for v in obj:
            out += find_acc_arrays(v)
    return out
accs = find_acc_arrays(d)
for a in accs:
    if a and isinstance(a[0], dict) and 'id' in a[0]:
        print('ACC ARRAY ids:', [x.get('id') for x in a][:25])
