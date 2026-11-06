"""Decode original Amiga four-plane tiles for development reference only (stdlib)."""
import pathlib,struct,zlib,json
ROOT=pathlib.Path(__file__).resolve().parents[1]
DIGITS=['111101101101111','010110010010111','111001111100111','111001111001111','101101111001001','111100111001111','111100111101111','111001001001001','111101111101111','111101111001111']
def png(path,w,h,pixels):
 def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
 path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(b'\0'+pixels[y*w*3:(y+1)*w*3] for y in range(h)),9))+chunk(b'IEND',b''))
def decode(directory,base,subset):
 pal=(directory/(base+'.pal')).read_bytes();palette=[tuple(((c>>shift)&15)*17 for shift in [8,4,0]) for (c,) in struct.iter_unpack('>H',pal)]
 data=(directory/(base+'.blk')).read_bytes()+(directory/(subset+'.blk')).read_bytes()
 return [[palette[sum(((struct.unpack_from('>H',data,i*128+plane*32+y*2)[0]>>(15-x))&1)<<plane for plane in range(4))] for y in range(16) for x in range(16)] for i in range(len(data)//128)]
def generate():
 directory=ROOT/'data/Data/AmigaTheOne';tiles=decode(directory,'junbase','junsub0');out=ROOT/'tmp';out.mkdir(exist_ok=True)
 w=800;h=((len(tiles)+19)//20)*44;pixels=bytearray([25]*(w*h*3))
 def put(x,y,c):pixels[(y*w+x)*3:(y*w+x)*3+3]=bytes(c)
 for i,tile in enumerate(tiles):
  ox=(i%20)*40;oy=(i//20)*44
  for y in range(32):
   for x in range(32):put(ox+x,oy+y,tile[y//2*16+x//2])
  for n,d in enumerate(str(i)):
   for j,bit in enumerate(DIGITS[int(d)]):
    if bit=='1':put(ox+n*5+j%3,oy+34+j//3,(255,240,200))
 png(out/'jungle-tiles.png',w,h,pixels)
 raw=(directory/'mapm1.map').read_bytes();mw,mh=struct.unpack_from('>HH',raw,84);pixels=bytearray(mw*16*mh*16*3)
 for i,(word,) in enumerate(struct.iter_unpack('>H',raw[96:96+mw*mh*2])):
  for y in range(16):
   for x in range(16):
    offset=(((i//mw)*16+y)*mw*16+(i%mw)*16+x)*3;pixels[offset:offset+3]=bytes(tiles[word&511][y*16+x])
 png(out/'original-first-mission.png',mw*16,mh*16,pixels)
 print('Reference images: tmp/jungle-tiles.png, tmp/original-first-mission.png')
if __name__=='__main__':generate()
