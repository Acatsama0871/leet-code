import { ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { LogOut, Github } from 'lucide-react';
import Button from './Button';
import { authApi } from '@/api/client';
import { User } from '@/types';

interface LayoutProps {
  children: ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const queryClient = useQueryClient();

  const { data: user } = useQuery<User>({
    queryKey: ['currentUser'],
    queryFn: authApi.getCurrentUser,
  });

  const handleLogout = async () => {
    await authApi.logout();
    localStorage.removeItem('session');
    queryClient.clear();
    window.location.href = '/';
  };

  return (
    <div className="min-h-screen bg-base-darker">
      {/* Header */}
      <header className="bg-base border-b border-surface-0 sticky top-0 z-10">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <h1 className="text-2xl font-bold text-mauve">LeetCode Tracker</h1>
            </div>

            {user && (
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <img
                    src={user.avatar_url}
                    alt={user.username}
                    className="w-8 h-8 rounded-full border-2 border-mauve"
                  />
                  <div className="flex items-center gap-1 text-text-muted">
                    <Github size={16} />
                    <span className="text-sm">{user.username}</span>
                  </div>
                </div>
                <Button
                  onClick={handleLogout}
                  variant="secondary"
                  size="sm"
                  className="flex items-center gap-2"
                >
                  <LogOut size={16} />
                  Logout
                </Button>
              </div>
            )}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8">{children}</main>
    </div>
  );
}
