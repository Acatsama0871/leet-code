export interface Question {
  question_number: number;
  problem: string;
  done: boolean;
  difficulty: string;
  tags: string;
}

export interface QuestionUpdate {
  done?: boolean;
  difficulty?: string;
}

export interface TagsUpdate {
  tags: string[];
}

export interface Tag {
  tag_name: string;
}

export interface CreateTag {
  tag_name: string;
}

export interface ListInfo {
  name: string;
  display_name: string;
  total_questions: number;
}

export interface IntersectionInfo {
  id: string;
  display_name: string;
  list1: string;
  list2: string;
}

export interface Metrics {
  total: number;
  completed: number;
  percentage: number;
}

export interface User {
  github_id: number;
  username: string;
  avatar_url: string;
}
