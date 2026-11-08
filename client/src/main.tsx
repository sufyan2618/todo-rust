import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter, Routes, Route, Navigate, Outlet } from 'react-router-dom';
import { useAuth } from './stores/auth';
import { AuthPage } from './pages/AuthPage';
import { TodosPage } from './pages/TodosPage';
import './styles.css';
function ProtectedRoute() { return useAuth((state) => state.token) ? <Outlet/> : <Navigate to="/login" replace/>; }
ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><BrowserRouter><Routes><Route path="/login" element={<AuthPage key="login" mode="login"/>}/><Route path="/signup" element={<AuthPage key="signup" mode="signup"/>}/><Route element={<ProtectedRoute/>}><Route path="/" element={<TodosPage/>}/></Route><Route path="*" element={<Navigate to="/" replace/>}/></Routes></BrowserRouter></React.StrictMode>);
