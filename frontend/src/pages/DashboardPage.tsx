import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { List, GitMerge, Tags } from 'lucide-react';
import Layout from '@/components/Layout';
import Card from '@/components/Card';
import { listsApi, intersectionsApi } from '@/api/client';
import ListsView from './ListsView';
import IntersectionsView from './IntersectionsView';
import TagsView from './TagsView';

type ViewType = 'lists' | 'intersections' | 'tags';

export default function DashboardPage() {
  const [activeView, setActiveView] = useState<ViewType>('lists');
  const [selectedItem, setSelectedItem] = useState<string>('');

  const { data: lists } = useQuery({
    queryKey: ['lists'],
    queryFn: listsApi.getLists,
  });

  const { data: intersections } = useQuery({
    queryKey: ['intersections'],
    queryFn: intersectionsApi.getIntersections,
  });

  const renderView = () => {
    switch (activeView) {
      case 'lists':
        return <ListsView listName={selectedItem} />;
      case 'intersections':
        return <IntersectionsView intersectionId={selectedItem} />;
      case 'tags':
        return <TagsView />;
      default:
        return null;
    }
  };

  return (
    <Layout>
      <div className="grid grid-cols-12 gap-6">
        {/* Sidebar */}
        <div className="col-span-12 lg:col-span-3">
          <Card className="space-y-6">
            {/* Lists Section */}
            <div>
              <button
                onClick={() => {
                  setActiveView('lists');
                  setSelectedItem(lists?.[0]?.name || '');
                }}
                className="flex items-center gap-2 text-mauve hover:text-mauve/80 font-semibold mb-3"
              >
                <List size={20} />
                <span>Lists</span>
              </button>
              <div className="space-y-1 ml-7">
                {lists?.map((list) => (
                  <button
                    key={list.name}
                    onClick={() => {
                      setActiveView('lists');
                      setSelectedItem(list.name);
                    }}
                    className={`block w-full text-left px-3 py-2 rounded text-sm transition-colors ${
                      activeView === 'lists' && selectedItem === list.name
                        ? 'bg-mauve/20 text-mauve'
                        : 'text-text-muted hover:bg-surface-1 hover:text-text'
                    }`}
                  >
                    {list.display_name}
                  </button>
                ))}
              </div>
            </div>

            {/* Intersections Section */}
            <div>
              <button
                onClick={() => {
                  setActiveView('intersections');
                  setSelectedItem(intersections?.[0]?.id || '');
                }}
                className="flex items-center gap-2 text-blue hover:text-blue/80 font-semibold mb-3"
              >
                <GitMerge size={20} />
                <span>Intersections</span>
              </button>
              <div className="space-y-1 ml-7">
                {intersections?.map((intersection) => (
                  <button
                    key={intersection.id}
                    onClick={() => {
                      setActiveView('intersections');
                      setSelectedItem(intersection.id);
                    }}
                    className={`block w-full text-left px-3 py-2 rounded text-sm transition-colors ${
                      activeView === 'intersections' && selectedItem === intersection.id
                        ? 'bg-blue/20 text-blue'
                        : 'text-text-muted hover:bg-surface-1 hover:text-text'
                    }`}
                  >
                    {intersection.display_name}
                  </button>
                ))}
              </div>
            </div>

            {/* Tags Section */}
            <div>
              <button
                onClick={() => {
                  setActiveView('tags');
                  setSelectedItem('');
                }}
                className={`flex items-center gap-2 font-semibold ${
                  activeView === 'tags'
                    ? 'text-green'
                    : 'text-text-muted hover:text-green'
                }`}
              >
                <Tags size={20} />
                <span>Manage Tags</span>
              </button>
            </div>
          </Card>
        </div>

        {/* Main Content */}
        <div className="col-span-12 lg:col-span-9">{renderView()}</div>
      </div>
    </Layout>
  );
}
