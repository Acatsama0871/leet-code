import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Trash2 } from 'lucide-react';
import Card from '@/components/Card';
import Button from '@/components/Button';
import Input from '@/components/Input';
import { tagsApi } from '@/api/client';

export default function TagsView() {
  const queryClient = useQueryClient();
  const [newTagName, setNewTagName] = useState('');

  const { data: tags } = useQuery({
    queryKey: ['tags'],
    queryFn: tagsApi.getTags,
  });

  const createTagMutation = useMutation({
    mutationFn: (tagName: string) => tagsApi.createTag({ tag_name: tagName }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tags'] });
      setNewTagName('');
    },
  });

  const deleteTagMutation = useMutation({
    mutationFn: (tagName: string) => tagsApi.deleteTag(tagName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tags'] });
    },
  });

  const handleCreateTag = (e: React.FormEvent) => {
    e.preventDefault();
    if (newTagName.trim()) {
      createTagMutation.mutate(newTagName.trim());
    }
  };

  return (
    <div className="space-y-6">
      <Card>
        <h2 className="text-2xl font-bold text-green mb-6">Manage Tags</h2>

        {/* Create New Tag */}
        <form onSubmit={handleCreateTag} className="flex gap-2 mb-6">
          <Input
            type="text"
            placeholder="New tag name..."
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            className="flex-1"
          />
          <Button type="submit" variant="primary" className="flex items-center gap-2">
            <Plus size={20} />
            Add Tag
          </Button>
        </form>

        {/* Tags List */}
        <div className="space-y-2">
          <h3 className="text-lg font-semibold text-text-muted mb-3">
            Existing Tags ({tags?.length || 0})
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
            {tags?.map((tag) => (
              <div
                key={tag.tag_name}
                className="flex items-center justify-between bg-surface-1 px-4 py-2 rounded-lg"
              >
                <span className="text-text">{tag.tag_name}</span>
                <button
                  onClick={() => {
                    if (confirm(`Delete tag "${tag.tag_name}"? This will remove it from all questions.`)) {
                      deleteTagMutation.mutate(tag.tag_name);
                    }
                  }}
                  className="text-red hover:text-red/80 transition-colors"
                >
                  <Trash2 size={18} />
                </button>
              </div>
            ))}
          </div>
          {(!tags || tags.length === 0) && (
            <p className="text-text-muted text-center py-8">
              No tags yet. Create one above to get started!
            </p>
          )}
        </div>
      </Card>
    </div>
  );
}
