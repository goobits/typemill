"""
Example backend Python file for CodeFlow Buddy testing
FastAPI application with user management
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional
import uvicorn

app = FastAPI(title="Backend API Example", version="1.0.0")

# Data models
class User(BaseModel):
    id: int
    name: str
    email: str
    is_active: bool = True

class UserCreate(BaseModel):
    name: str
    email: str

# In-memory storage for demo
users_db: List[User] = [
    User(id=1, name="John Doe", email="john@example.com"),
    User(id=2, name="Jane Smith", email="jane@example.com"),
]

@app.get("/")
async def root():
    """Root endpoint"""
    return {"message": "CodeFlow Buddy Backend API"}

@app.get("/users", response_model=List[User])
async def get_users():
    """Get all users"""
    return users_db

@app.get("/users/{user_id}", response_model=User)
async def get_user(user_id: int):
    """Get user by ID"""
    user = find_user_by_id(user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    return user

@app.post("/users", response_model=User)
async def create_user(user_data: UserCreate):
    """Create a new user"""
    new_id = max([u.id for u in users_db], default=0) + 1
    new_user = User(id=new_id, **user_data.dict())
    users_db.append(new_user)
    return new_user

def find_user_by_id(user_id: int) -> Optional[User]:
    """Helper function to find user by ID"""
    for user in users_db:
        if user.id == user_id:
            return user
    return None

def calculate_user_score(user: User) -> float:
    """Sample function for testing LSP features"""
    base_score = len(user.name) * 10
    email_bonus = 5 if "@" in user.email else 0
    active_bonus = 20 if user.is_active else 0
    return float(base_score + email_bonus + active_bonus)

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)