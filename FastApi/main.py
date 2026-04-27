from fastapi import FastAPI,Depends,HTTPException
from fastapi.responses import RedirectResponse
from sqlalchemy.orm import Session
import random
import string

from database import engine,Base,SessionLocal
from models import URL
from schemas import URLCreate,URLResponse

app = FastAPI(title = "URL shortener API")

Base.metadata.create_all(bind=engine)

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

def generate_short_code(length:int = 6):
    characters = string.ascii_letters + string.digits
    return "".join(random.choice(characters) for _ in range(length))


@app.get("/")
async def root():
    return {"message":"URL Shortener API is running"}


@app.post("/shorten",response_model = URLResponse)
def create_short_url(url_data:URLCreate,db:Session = Depends(get_db)):
    short_code = generate_short_code()

    new_url = URL(
        original_url = str(url_data.original_url),
        short_code = short_code
    )

    db.add(new_url)
    db.commit()
    db.refresh(new_url)

    return new_url


@app.get("/{short_code}")
def redirect_to_original_url(short_code:str,db:Session = Depends(get_db)):
    url_record = db.query(URL).filter(URL.short_code == short_code).first()

    if not url_record:
        raise HTTPException(status_code = 404,details = "Short URL not found")
    
    url_record.clicks +=1
    db.commit()

    return RedirectResponse(url=url_record.original_url)