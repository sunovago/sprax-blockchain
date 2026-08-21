import React, { useState } from 'react';
import { Search, ChevronLeft, ChevronRight } from 'lucide-react';

export interface Column<T> {
  header: string;
  accessorKey?: keyof T;
  cell?: (item: T) => React.ReactNode;
  className?: string;
}

interface DataTableProps<T> {
  data: T[];
  columns: Column<T>[];
  searchPlaceholder?: string;
  onSearch?: (term: string) => void;
  isLoading?: boolean;
  emptyMessage?: string;
  pageSize?: number;
  onRowClick?: (item: T) => void;
}

export function DataTable<T extends Record<string, any>>({
  data,
  columns,
  searchPlaceholder = 'Search records...',
  isLoading = false,
  emptyMessage = 'No records found.',
  pageSize = 10,
  onRowClick,
}: DataTableProps<T>) {
  const [searchTerm, setSearchTerm] = useState('');
  const [currentPage, setCurrentPage] = useState(1);

  const filteredData = data.filter((item) => {
    if (!searchTerm) return true;
    return Object.values(item).some(
      (val) => val && String(val).toLowerCase().includes(searchTerm.toLowerCase())
    );
  });

  const totalPages = Math.ceil(filteredData.length / pageSize) || 1;
  const paginatedData = filteredData.slice((currentPage - 1) * pageSize, currentPage * pageSize);

  return (
    <div className="bg-[#111827] border border-[#1E293B] rounded-xl overflow-hidden shadow-sm">
      {/* Search and filter bar */}
      <div className="p-4 border-b border-[#1E293B] flex items-center justify-between gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
          <input
            type="text"
            placeholder={searchPlaceholder}
            value={searchTerm}
            onChange={(e) => {
              setSearchTerm(e.target.value);
              setCurrentPage(1);
            }}
            className="w-full pl-9 pr-4 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white placeholder-gray-500 focus:outline-none focus:border-primary-500 transition-colors"
          />
        </div>
        <div className="text-xs text-gray-400">
          Showing <span className="text-white font-medium">{filteredData.length}</span> total entries
        </div>
      </div>

      {/* Table container */}
      <div className="overflow-x-auto">
        <table className="w-full text-left text-xs">
          <thead className="bg-[#0B0F19]/50 text-gray-400 border-b border-[#1E293B] font-semibold uppercase tracking-wider">
            <tr>
              {columns.map((col, idx) => (
                <th key={idx} className={`px-4 py-3 ${col.className || ''}`}>
                  {col.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-[#1E293B]">
            {isLoading ? (
              <tr>
                <td colSpan={columns.length} className="px-4 py-12 text-center text-gray-400">
                  <div className="inline-flex items-center gap-2">
                    <div className="w-4 h-4 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
                    Loading data...
                  </div>
                </td>
              </tr>
            ) : paginatedData.length === 0 ? (
              <tr>
                <td colSpan={columns.length} className="px-4 py-12 text-center text-gray-500">
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              paginatedData.map((item, rowIdx) => (
                <tr
                  key={rowIdx}
                  onClick={() => onRowClick && onRowClick(item)}
                  className={`hover:bg-[#1E293B]/40 transition-colors ${onRowClick ? 'cursor-pointer' : ''}`}
                >
                  {columns.map((col, colIdx) => (
                    <td key={colIdx} className={`px-4 py-3 text-gray-300 ${col.className || ''}`}>
                      {col.cell ? col.cell(item) : col.accessorKey ? String(item[col.accessorKey] ?? '') : ''}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination controls */}
      <div className="p-3 border-t border-[#1E293B] flex items-center justify-between text-xs text-gray-400 bg-[#0B0F19]/30">
        <div>
          Page {currentPage} of {totalPages}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="p-1.5 rounded-lg border border-[#1E293B] hover:bg-[#1E293B] disabled:opacity-40 disabled:cursor-not-allowed text-gray-300 transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <button
            onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className="p-1.5 rounded-lg border border-[#1E293B] hover:bg-[#1E293B] disabled:opacity-40 disabled:cursor-not-allowed text-gray-300 transition-colors"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}
