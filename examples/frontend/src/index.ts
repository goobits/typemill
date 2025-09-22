// Example frontend TypeScript file for CodeFlow Buddy testing
import React from 'react';

export interface User {
  id: number;
  name: string;
  email: string;
}

export const UserComponent: React.FC<{ user: User }> = ({ user }) => {
  return React.createElement('div', null, `Hello, ${user.name}!`);
};

export function fetchUser(id: number): Promise<User> {
  return fetch(`/api/users/${id}`)
    .then((response) => response.json())
    .then((data) => data as User);
}

// This is a sample function for testing LSP features
export function calculateTotal(items: Array<{ price: number; quantity: number }>): number {
  return items.reduce((total, item) => total + item.price * item.quantity, 0);
}
