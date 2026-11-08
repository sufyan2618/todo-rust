import { useState, type FormEvent } from 'react';
import { Link, Navigate, useNavigate } from 'react-router-dom';
import { ArrowRight, Check, Layers, Sparkles } from 'lucide-react';
import { Brand } from '../components/Brand';
import { useAuth } from '../stores/auth';
import { errorMessage } from '../lib/api';
export function AuthPage({ mode }: { mode: 'login' | 'signup' }) {
  const signup = mode === 'signup';
  const { token, authenticate } = useAuth();
  const navigate = useNavigate();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  if (token) return <Navigate to="/" replace/>;
  async function submit(event: FormEvent) {
    event.preventDefault(); setBusy(true); setError('');
    try { await authenticate(mode, { name: name.trim(), email: email.trim(), password }); navigate('/', { replace: true }); }
    catch (error) { setError(errorMessage(error)); }
    finally { setBusy(false); }
  }
  return <main className="auth-layout"><section className="auth-story"><Brand/><div className="story-content"><span className="eyebrow"><Sparkles size={14}/> A LITTLE SPACE TO FOCUS</span><h1>Big days start<br/>with small <em>steps.</em></h1><p>A calmer place for everything you want to get done. Clear your mind. Find your focus. Make it happen.</p><div className="sample-card"><span className="sample-label">A FRESH START</span><div><span className="sample-check"><Check size={13}/></span>Make room for what matters</div><div><span className="sample-check"><Check size={13}/></span>Take it one task at a time</div><div><span className="sample-empty"/>Do something great today</div><span className="sample-footer"><Layers size={14}/> Less noise. More progress.</span></div></div><span className="story-footer">A little more organized. A lot more you.</span></section><section className="auth-panel"><div className="auth-form-wrap"><span className="page-kicker">YOUR DAY, WITH A LITTLE MORE CLARITY</span><h2>{signup ? 'Make yourself at home.' : 'Welcome back.'}</h2><p className="muted">{signup ? 'Create an account and start making space for what matters.' : 'Your next small step is waiting for you.'}</p><form onSubmit={submit} className="auth-form">{signup && <label>Your name<input autoComplete="name" required maxLength={80} placeholder="Alex Morgan" value={name} onChange={(e) => setName(e.target.value)}/></label>}<label>Email address<input type="email" autoComplete="email" required maxLength={254} placeholder="you@example.com" value={email} onChange={(e) => setEmail(e.target.value)}/></label><label>Password<input type="password" autoComplete={signup ? 'new-password' : 'current-password'} minLength={signup ? 8 : 1} maxLength={128} required placeholder={signup ? 'At least 8 characters' : 'Enter your password'} value={password} onChange={(e) => setPassword(e.target.value)}/></label>{error && <p className="error" role="alert">{error}</p>}<button className="primary-button" disabled={busy}>{busy ? 'One moment…' : signup ? 'Create account' : 'Sign in'}<ArrowRight size={17}/></button></form><p className="switch-auth">{signup ? 'Already have an account?' : 'New around here?'} <Link to={signup ? '/login' : '/signup'}>{signup ? 'Sign in' : 'Create an account'}</Link></p><p className="auth-note">Your tasks. Your space. A fresh perspective.</p></div></section></main>;
}
