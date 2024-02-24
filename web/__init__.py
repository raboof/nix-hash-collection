from fastapi import Depends, FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from sqlalchemy.orm import Session

from . import models, schemas, crud
from .db import SessionLocal, engine

models.Base.metadata.create_all(bind=engine)

app = FastAPI()

origins = [
    "http://localhost:8000",
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


def get_drv_or_404(session, drv_hash):
    o = session.query(models.Derivation).filter_by(drv_hash=drv_hash).one_or_none()
    if o is None:
        raise HTTPException(status_code=404, detail="Not found")

    return o


@app.get("/derivations/")
def get_derivations(db: Session = Depends(get_db)):
    return db.query(models.Derivation).all()


@app.get("/derivation/{drv_hash}")
def get_machines(drv_hash: str, db: Session = Depends(get_db)):
    return get_drv_or_404(db, drv_hash)


@app.post("/report/{drv_hash}")
def record_report(
    drv_hash: str,
    token: str,
    output_sha256_map: list[schemas.OuputHashPair],
    db: Session = Depends(get_db),
):
    user = crud.get_user_with_token(db, token)
    crud.create_report(db, drv_hash, output_sha256_map, user)
    return {
        "message": f"hello"
    }


