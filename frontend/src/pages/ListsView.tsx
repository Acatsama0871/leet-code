import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import Card from '@/components/Card';
import Button from '@/components/Button';
import Select from '@/components/Select';
import { listsApi, questionsApi, tagsApi } from '@/api/client';
import { Question } from '@/types';
import { Edit2, Save, X } from 'lucide-react';

interface ListsViewProps {
  listName: string;
}

export default function ListsView({ listName }: ListsViewProps) {
  const queryClient = useQueryClient();
  const [editingQuestion, setEditingQuestion] = useState<number | null>(null);
  const [editingTags, setEditingTags] = useState<string[]>([]);

  const { data: questions, isLoading } = useQuery({
    queryKey: ['listQuestions', listName],
    queryFn: () => listsApi.getListQuestions(listName),
    enabled: !!listName,
  });

  const { data: metrics } = useQuery({
    queryKey: ['metrics', listName],
    queryFn: () => listsApi.getMetrics(listName),
    enabled: !!listName,
  });

  const { data: allTags } = useQuery({
    queryKey: ['tags'],
    queryFn: tagsApi.getTags,
  });

  const updateQuestionMutation = useMutation({
    mutationFn: ({ questionNumber, update }: { questionNumber: number; update: any }) =>
      questionsApi.updateQuestion(questionNumber, update),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['listQuestions', listName] });
      queryClient.invalidateQueries({ queryKey: ['metrics', listName] });
    },
  });

  const updateTagsMutation = useMutation({
    mutationFn: ({ questionNumber, tags }: { questionNumber: number; tags: string[] }) =>
      questionsApi.updateQuestionTags(questionNumber, { tags }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['listQuestions', listName] });
      setEditingQuestion(null);
    },
  });

  const handleDoneChange = (question: Question) => {
    updateQuestionMutation.mutate({
      questionNumber: question.question_number,
      update: { done: !question.done },
    });
  };

  const handleDifficultyChange = (question: Question, difficulty: string) => {
    updateQuestionMutation.mutate({
      questionNumber: question.question_number,
      update: { difficulty },
    });
  };

  const handleEditTags = (question: Question) => {
    setEditingQuestion(question.question_number);
    setEditingTags(question.tags ? question.tags.split('; ').filter(Boolean) : []);
  };

  const handleSaveTags = (questionNumber: number) => {
    updateTagsMutation.mutate({ questionNumber, tags: editingTags });
  };

  const handleCancelEdit = () => {
    setEditingQuestion(null);
    setEditingTags([]);
  };

  const toggleTag = (tag: string) => {
    setEditingTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
    );
  };

  if (!listName) {
    return (
      <Card>
        <p className="text-text-muted text-center">Select a list to view questions</p>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card>
        <p className="text-text-muted text-center">Loading...</p>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Metrics */}
      {metrics && (
        <Card>
          <div className="grid grid-cols-3 gap-4 text-center">
            <div>
              <p className="text-3xl font-bold text-mauve">{metrics.total}</p>
              <p className="text-sm text-text-muted">Total Questions</p>
            </div>
            <div>
              <p className="text-3xl font-bold text-green">{metrics.completed}</p>
              <p className="text-sm text-text-muted">Completed</p>
            </div>
            <div>
              <p className="text-3xl font-bold text-blue">{metrics.percentage.toFixed(1)}%</p>
              <p className="text-sm text-text-muted">Progress</p>
            </div>
          </div>
        </Card>
      )}

      {/* Questions Table */}
      <Card>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-surface-1">
                <th className="table-header px-4 py-3 text-left w-16">Done</th>
                <th className="table-header px-4 py-3 text-left w-20">#</th>
                <th className="table-header px-4 py-3 text-left">Problem</th>
                <th className="table-header px-4 py-3 text-left w-32">Difficulty</th>
                <th className="table-header px-4 py-3 text-left w-48">Tags</th>
                <th className="table-header px-4 py-3 text-left w-24">Actions</th>
              </tr>
            </thead>
            <tbody>
              {questions?.map((question) => (
                <tr key={question.question_number} className="table-row">
                  <td className="px-4 py-3">
                    <input
                      type="checkbox"
                      checked={question.done}
                      onChange={() => handleDoneChange(question)}
                      className="w-5 h-5 rounded border-surface-1 bg-surface-0 text-mauve focus:ring-2 focus:ring-mauve cursor-pointer"
                    />
                  </td>
                  <td className="px-4 py-3 text-text-muted">{question.question_number}</td>
                  <td className="px-4 py-3 text-text">{question.problem}</td>
                  <td className="px-4 py-3">
                    <Select
                      value={question.difficulty}
                      onChange={(e) => handleDifficultyChange(question, e.target.value)}
                      className="text-sm py-1"
                    >
                      <option value="">-</option>
                      <option value="Easy" className="text-green">
                        Easy
                      </option>
                      <option value="Medium" className="text-yellow">
                        Medium
                      </option>
                      <option value="Hard" className="text-red">
                        Hard
                      </option>
                    </Select>
                  </td>
                  <td className="px-4 py-3">
                    {editingQuestion === question.question_number ? (
                      <div className="space-y-2">
                        <div className="flex flex-wrap gap-1">
                          {allTags?.map((tag) => (
                            <button
                              key={tag.tag_name}
                              onClick={() => toggleTag(tag.tag_name)}
                              className={`px-2 py-1 rounded text-xs ${
                                editingTags.includes(tag.tag_name)
                                  ? 'bg-mauve text-base'
                                  : 'bg-surface-1 text-text-muted'
                              }`}
                            >
                              {tag.tag_name}
                            </button>
                          ))}
                        </div>
                      </div>
                    ) : (
                      <span className="text-sm text-text-muted">
                        {question.tags || '-'}
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    {editingQuestion === question.question_number ? (
                      <div className="flex gap-1">
                        <Button
                          size="sm"
                          variant="primary"
                          onClick={() => handleSaveTags(question.question_number)}
                          className="p-1"
                        >
                          <Save size={16} />
                        </Button>
                        <Button
                          size="sm"
                          variant="secondary"
                          onClick={handleCancelEdit}
                          className="p-1"
                        >
                          <X size={16} />
                        </Button>
                      </div>
                    ) : (
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => handleEditTags(question)}
                        className="p-1"
                      >
                        <Edit2 size={16} />
                      </Button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
