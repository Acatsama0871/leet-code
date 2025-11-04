import { Github } from 'lucide-react';
import Button from '@/components/Button';
import Card from '@/components/Card';
import { authApi } from '@/api/client';

export default function LoginPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-base-darker p-4">
      <Card className="max-w-md w-full">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-mauve mb-2">LeetCode Tracker</h1>
          <p className="text-text-muted">Track your progress across multiple lists</p>
        </div>

        <div className="space-y-4">
          <Button
            onClick={() => authApi.loginWithGithub()}
            variant="primary"
            className="w-full flex items-center justify-center gap-2"
            size="lg"
          >
            <Github size={20} />
            Sign in with GitHub
          </Button>

          <p className="text-sm text-text-subtle text-center">
            Sign in to track your LeetCode progress and manage your question lists
          </p>
        </div>
      </Card>
    </div>
  );
}
