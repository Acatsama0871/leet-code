import axios from 'axios';
import type {
  Question,
  QuestionUpdate,
  TagsUpdate,
  Tag,
  CreateTag,
  ListInfo,
  IntersectionInfo,
  Metrics,
  User,
} from '@/types';

const api = axios.create({
  baseURL: '/api',
  withCredentials: true,
});

// Add session to requests
api.interceptors.request.use((config) => {
  const session = localStorage.getItem('session');
  if (session) {
    config.params = { ...config.params, session };
  }
  return config;
});

// Auth API
export const authApi = {
  getCurrentUser: () => api.get<User>('/auth/me').then((res) => res.data),
  logout: () => api.post('/auth/logout').then((res) => res.data),
  loginWithGithub: () => {
    window.location.href = '/api/auth/github';
  },
};

// Lists API
export const listsApi = {
  getLists: () => api.get<ListInfo[]>('/lists').then((res) => res.data),
  getListQuestions: (listName: string) =>
    api.get<Question[]>(`/lists/${listName}`).then((res) => res.data),
  getMetrics: (listName: string) =>
    api.get<Metrics>(`/metrics/${listName}`).then((res) => res.data),
};

// Intersections API
export const intersectionsApi = {
  getIntersections: () =>
    api.get<IntersectionInfo[]>('/intersections').then((res) => res.data),
  getIntersectionQuestions: (intersectionId: string) =>
    api.get<Question[]>(`/intersections/${intersectionId}`).then((res) => res.data),
};

// Questions API
export const questionsApi = {
  updateQuestion: (questionNumber: number, update: QuestionUpdate) =>
    api.put(`/questions/${questionNumber}`, update).then((res) => res.data),
  getQuestionTags: (questionNumber: number) =>
    api.get<string[]>(`/questions/${questionNumber}/tags`).then((res) => res.data),
  updateQuestionTags: (questionNumber: number, update: TagsUpdate) =>
    api.put(`/questions/${questionNumber}/tags`, update).then((res) => res.data),
};

// Tags API
export const tagsApi = {
  getTags: () => api.get<Tag[]>('/tags').then((res) => res.data),
  createTag: (tag: CreateTag) => api.post('/tags', tag).then((res) => res.data),
  deleteTag: (tagName: string) => api.delete(`/tags/${tagName}`).then((res) => res.data),
};
