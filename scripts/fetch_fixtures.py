"""Fetch unmodified OpenFodder fixtures; pin and verify every downloaded byte."""
import hashlib,json,pathlib,urllib.request,concurrent.futures
REV='3feb494023e4d008f8d7b548167e3e9ad31bad29'
ROOT=pathlib.Path(__file__).resolve().parents[1]
manifest=ROOT/'tests/fixtures-manifest.json'
def fetch(path):
    url=f'https://raw.githubusercontent.com/OpenFodder/data/{REV}/{urllib.parse.quote(path)}'
    raw=urllib.request.urlopen(url,timeout=30).read()
    dst=ROOT/'data'/path; dst.parent.mkdir(parents=True,exist_ok=True)
    digest=hashlib.sha256(raw).hexdigest()
    if manifest.exists():
        expected=json.loads(manifest.read_text())['files'][path]
        if digest!=expected: raise RuntimeError(f'Checksum mismatch: {path}')
    dst.write_bytes(raw)
    return path,digest
if __name__=='__main__':
    import urllib.parse
    if manifest.exists(): paths=list(json.loads(manifest.read_text())['files'])
    else:
        tree=json.load(urllib.request.urlopen(f'https://api.github.com/repos/OpenFodder/data/git/trees/{REV}?recursive=1'))['tree']
        paths=[x['path'] for x in tree if x['type']=='blob' and (x['path'].startswith(('Data/AmigaTheOne/','Data/AmigaAction/','Data/Plus/','Data/Custom/Maps/')) and x['path'].endswith(('.map','.spt','.hit','.bht','.swp')) or x['path'].startswith('Campaigns/') and x['path'].endswith('.ofc'))]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool: hashes=dict(pool.map(fetch,paths))
    if not manifest.exists(): manifest.write_text(json.dumps({'revision':REV,'files':hashes},indent=2)+'\n')
    print(f'Verified {len(hashes)} files from {REV}')
